use std::path::Path;

use crate::config::read_config;
use crate::hooks::{HookItemType, HookOperation};
// Domain function accessed via fully-qualified path to avoid name conflict with handler
use crate::registry::track_project_async;
use crate::server::convert_entity::pr_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{SoftDeletePrRequest, SoftDeletePrResponse};
use crate::server::resolve::resolve_pr_id;
use tonic::{Response, Status};

pub async fn soft_delete_pr(
    req: SoftDeletePrRequest,
) -> Result<Response<SoftDeletePrResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.pr_id.clone();
    let hook_request_data = serde_json::json!({
        "pr_id": &req.pr_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Pr,
        HookOperation::SoftDelete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(SoftDeletePrResponse {
            success: false,
            error: e,
            ..Default::default()
        }));
    }

    // Read config for priority_levels
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    let pr_id = match resolve_pr_id(project_path, &req.pr_id).await {
        Ok(id) => id,
        Err(e) => {
            return Ok(Response::new(SoftDeletePrResponse {
                success: false,
                error: e,
                ..Default::default()
            }))
        }
    };

    match crate::item::entities::pr::soft_delete_pr(project_path, &pr_id).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Pr,
                HookOperation::SoftDelete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            Ok(Response::new(SoftDeletePrResponse {
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
                HookOperation::SoftDelete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(SoftDeletePrResponse {
                success: false,
                error: e.to_string(),
                pr: None,
                manifest: None,
            }))
        }
    }
}
