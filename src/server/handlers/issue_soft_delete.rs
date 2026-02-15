use std::path::Path;

use crate::config::read_config;
use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::issue::get_issue;
use crate::item::generic::storage::generic_soft_delete;
use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::convert_entity::issue_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::handlers::item_type_resolve::resolve_item_type_config;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{SoftDeleteIssueRequest, SoftDeleteIssueResponse};
use crate::server::resolve::resolve_issue_id;
use crate::server::structured_error::to_error_json;
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
            error: to_error_json(&req.project_path, &e),
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
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };

    let item_config = match resolve_item_type_config(project_path, "issues").await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Response::new(SoftDeleteIssueResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };

    match generic_soft_delete(project_path, &item_config, &issue_id).await {
        Ok(()) => {
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

            // Read the issue back for the type-specific response
            let issue = get_issue(project_path, &issue_id).await.ok();
            let manifest = read_manifest(project_path).await.ok().flatten();

            Ok(Response::new(SoftDeleteIssueResponse {
                success: true,
                error: String::new(),
                issue: issue.map(|i| issue_to_proto(&i, priority_levels)),
                manifest: manifest.map(|m| manifest_to_proto(&m)),
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
                error: to_error_json(&req.project_path, &e),
                issue: None,
                manifest: None,
            }))
        }
    }
}
