use std::path::Path;

use crate::config::read_config;
use crate::hooks::HookOperation;
use crate::item::entities::issue::get_issue;
use crate::item::generic::storage::generic_move;
use crate::manifest;
use crate::registry::track_project_async;
use crate::server::convert_entity::issue_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{MoveIssueRequest, MoveIssueResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn move_issue(req: MoveIssueRequest) -> Result<Response<MoveIssueResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());

    let source_path = Path::new(&req.source_project_path);
    let target_path = Path::new(&req.target_project_path);

    // Pre-hook
    let hook_project_path = req.source_project_path.clone();
    let hook_item_id = req.issue_id.clone();
    let hook_request_data = serde_json::json!({
        "source_project_path": &req.source_project_path,
        "target_project_path": &req.target_project_path,
        "issue_id": &req.issue_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        Path::new(&hook_project_path),
        "issue",
        HookOperation::Move,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(MoveIssueResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }

    // Resolve configs for issues
    let source_config = match resolve_item_type_config(source_path, "issues").await {
        Ok((_type, config)) => config,
        Err(e) => {
            return Ok(Response::new(MoveIssueResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                ..Default::default()
            }));
        }
    };
    let target_config = match resolve_item_type_config(target_path, "issues").await {
        Ok((_type, config)) => config,
        Err(e) => {
            return Ok(Response::new(MoveIssueResponse {
                success: false,
                error: to_error_json(&req.target_project_path, &e),
                ..Default::default()
            }));
        }
    };

    // Read source display_number before move for response
    let old_display_number = match get_issue(source_path, &req.issue_id).await {
        Ok(issue) => issue.metadata.display_number,
        Err(e) => {
            return Ok(Response::new(MoveIssueResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                ..Default::default()
            }));
        }
    };

    match generic_move(
        source_path,
        target_path,
        &source_config,
        &target_config,
        &req.issue_id,
        None,
    )
    .await
    {
        Ok(_result) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                "issue",
                HookOperation::Move,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            // Read the moved issue via entity-specific reader for proto conversion
            let target_config_for_priority = read_config(target_path).await.ok().flatten();
            let priority_levels = target_config_for_priority
                .as_ref()
                .map_or(3, |c| c.priority_levels);

            let moved_issue = match get_issue(target_path, &req.issue_id).await {
                Ok(issue) => issue,
                Err(e) => {
                    return Ok(Response::new(MoveIssueResponse {
                        success: false,
                        error: to_error_json(&req.target_project_path, &e),
                        ..Default::default()
                    }));
                }
            };

            // Read manifests
            let source_manifest = manifest::read_manifest(source_path).await.ok().flatten();
            let target_manifest = manifest::read_manifest(target_path).await.ok().flatten();

            Ok(Response::new(MoveIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&moved_issue, priority_levels)),
                old_display_number,
                source_manifest: source_manifest.map(|m| manifest_to_proto(&m)),
                target_manifest: target_manifest.map(|m| manifest_to_proto(&m)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                "issue",
                HookOperation::Move,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(MoveIssueResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                issue: None,
                old_display_number: 0,
                source_manifest: None,
                target_manifest: None,
            }))
        }
    }
}
