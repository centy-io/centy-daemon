use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
// Domain function accessed via fully-qualified path to avoid name conflict with handler
use crate::registry::track_project_async;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DeletePrRequest, DeletePrResponse};
use crate::server::resolve::resolve_pr_id;
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn delete_pr(req: DeletePrRequest) -> Result<Response<DeletePrResponse>, Status> {
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
        HookOperation::Delete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DeletePrResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let pr_id = match resolve_pr_id(project_path, &req.pr_id).await {
        Ok(id) => id,
        Err(e) => {
            return Ok(Response::new(DeletePrResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };

    match crate::item::entities::pr::delete_pr(project_path, &pr_id).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Pr,
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            Ok(Response::new(DeletePrResponse {
                success: true,
                error: String::new(),
                manifest: Some(manifest_to_proto(&result.manifest)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Pr,
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(DeletePrResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                manifest: None,
            }))
        }
    }
}
