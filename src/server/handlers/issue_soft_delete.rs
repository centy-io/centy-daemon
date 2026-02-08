use std::path::Path;

use crate::config::read_config;
use crate::hooks::{HookItemType, HookOperation};
use crate::registry::track_project_async;
use crate::server::convert_entity::issue_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{SoftDeleteIssueRequest, SoftDeleteIssueResponse};
use crate::server::resolve::resolve_issue_id;
use tonic::{Response, Status};

pub async fn soft_delete_issue(
    req: SoftDeleteIssueRequest,
) -> Result<Response<SoftDeleteIssueResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.issue_id.clone();
    let hook_request_data = serde_json::json!({
        "issue_id": &req.issue_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Issue,
        HookOperation::SoftDelete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(SoftDeleteIssueResponse {
            success: false,
            error: e,
            ..Default::default()
        }));
    }

    // Read config for priority_levels
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    let issue_id = match resolve_issue_id(project_path, &req.issue_id).await {
        Ok(id) => id,
        Err(e) => {
            return Ok(Response::new(SoftDeleteIssueResponse {
                success: false,
                error: e,
                ..Default::default()
            }))
        }
    };

    match crate::item::entities::issue::soft_delete_issue(project_path, &issue_id).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Issue,
                HookOperation::SoftDelete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            Ok(Response::new(SoftDeleteIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&result.issue, priority_levels)),
                manifest: Some(manifest_to_proto(&result.manifest)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Issue,
                HookOperation::SoftDelete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(SoftDeleteIssueResponse {
                success: false,
                error: e.to_string(),
                issue: None,
                manifest: None,
            }))
        }
    }
}
