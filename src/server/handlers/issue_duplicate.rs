use std::path::{Path, PathBuf};

use crate::config::read_config;
use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::issue::DuplicateIssueOptions;
use crate::registry::track_project_async;
use crate::server::convert_entity::issue_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::nonempty;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DuplicateIssueRequest, DuplicateIssueResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn duplicate_issue(
    req: DuplicateIssueRequest,
) -> Result<Response<DuplicateIssueResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());
    let hook_project_path = req.source_project_path.clone();
    let hook_item_id = req.issue_id.clone();
    let hook_request_data = serde_json::json!({
        "source_project_path": &req.source_project_path,
        "target_project_path": &req.target_project_path,
        "issue_id": &req.issue_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        Path::new(&hook_project_path),
        HookItemType::Issue,
        HookOperation::Duplicate,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DuplicateIssueResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }

    let target_config = read_config(Path::new(&req.target_project_path))
        .await
        .ok()
        .flatten();
    let priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);

    let options = DuplicateIssueOptions {
        source_project_path: PathBuf::from(&req.source_project_path),
        target_project_path: PathBuf::from(&req.target_project_path),
        issue_id: req.issue_id,
        new_title: nonempty(req.new_title),
    };

    match crate::item::entities::issue::duplicate_issue(options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                HookItemType::Issue,
                HookOperation::Duplicate,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(DuplicateIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&result.issue, priority_levels)),
                original_issue_id: result.original_issue_id,
                manifest: Some(manifest_to_proto(&result.manifest)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                HookItemType::Issue,
                HookOperation::Duplicate,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(DuplicateIssueResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                issue: None,
                original_issue_id: String::new(),
                manifest: None,
            }))
        }
    }
}
