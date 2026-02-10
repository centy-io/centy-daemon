use std::path::Path;

use crate::config::read_config;
use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::pr::UpdatePrOptions;
use crate::registry::track_project_async;
use crate::server::convert_entity::pr_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::{nonempty, nonzero_u32};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{UpdatePrRequest, UpdatePrResponse};
use crate::server::resolve::resolve_pr_id;
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn update_pr(req: UpdatePrRequest) -> Result<Response<UpdatePrResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.pr_id.clone();
    let hook_request_data = serde_json::json!({
        "pr_id": &req.pr_id,
        "title": &req.title,
        "description": &req.description,
        "status": &req.status,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Pr,
        HookOperation::Update,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(UpdatePrResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    // Read config for priority_levels (for label generation)
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    let options = UpdatePrOptions {
        title: nonempty(req.title),
        description: nonempty(req.description),
        status: nonempty(req.status),
        source_branch: nonempty(req.source_branch),
        target_branch: nonempty(req.target_branch),
        reviewers: if req.reviewers.is_empty() {
            None
        } else {
            Some(req.reviewers)
        },
        priority: nonzero_u32(req.priority),
        custom_fields: req.custom_fields,
    };

    let pr_id = match resolve_pr_id(project_path, &req.pr_id).await {
        Ok(id) => id,
        Err(e) => {
            return Ok(Response::new(UpdatePrResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };

    match crate::item::entities::pr::update_pr(project_path, &pr_id, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Pr,
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(UpdatePrResponse {
                success: true,
                error: String::new(),
                pr: Some(pr_to_proto(&result.pr, priority_levels)),
                manifest: Some(manifest_to_proto(&result.manifest)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Pr,
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(UpdatePrResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                pr: None,
                manifest: None,
            }))
        }
    }
}
