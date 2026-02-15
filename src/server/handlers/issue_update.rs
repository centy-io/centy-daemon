use std::path::Path;

use crate::config::read_config;
use crate::hooks::HookOperation;
use crate::item::entities::issue::UpdateIssueOptions;
use crate::registry::track_project_async;
use crate::server::convert_entity::issue_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::{nonempty, nonzero_u32, sync_results_to_proto};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{UpdateIssueRequest, UpdateIssueResponse};
use crate::server::resolve::resolve_issue_id;
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn update_issue(
    req: UpdateIssueRequest,
) -> Result<Response<UpdateIssueResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.issue_id.clone();
    let hook_data = serde_json::json!({
        "issue_id": &req.issue_id, "title": &req.title,
        "description": &req.description, "priority": req.priority, "status": &req.status,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "issue",
        HookOperation::Update,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_data.clone()),
    )
    .await
    {
        return Ok(Response::new(UpdateIssueResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    let options = UpdateIssueOptions {
        title: nonempty(req.title),
        description: nonempty(req.description),
        status: nonempty(req.status),
        priority: nonzero_u32(req.priority),
        custom_fields: req.custom_fields,
        draft: req.draft,
    };

    let issue_id = match resolve_issue_id(project_path, &req.issue_id).await {
        Ok(id) => id,
        Err(e) => {
            return Ok(Response::new(UpdateIssueResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };

    match crate::item::entities::issue::update_issue(project_path, &issue_id, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                "issue",
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_data),
                true,
            )
            .await;
            Ok(Response::new(UpdateIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&result.issue, priority_levels)),
                manifest: Some(manifest_to_proto(&result.manifest)),
                sync_results: sync_results_to_proto(result.sync_results),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                "issue",
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_data),
                false,
            )
            .await;
            Ok(Response::new(UpdateIssueResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                issue: None,
                manifest: None,
                sync_results: vec![],
            }))
        }
    }
}
