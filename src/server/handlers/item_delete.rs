use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_delete;
use crate::registry::track_project_async;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DeleteItemRequest, DeleteItemResponse};
use crate::server::structured_error::to_error_json;
use crate::user::delete_user;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn delete_item(req: DeleteItemRequest) -> Result<Response<DeleteItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Route user type to user-specific handler
    let lower = req.item_type.to_lowercase();
    if lower == "user" || lower == "users" {
        return match delete_user(project_path, &req.item_id).await {
            Ok(_) => Ok(Response::new(DeleteItemResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(DeleteItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
            })),
        };
    }

    // Resolve config
    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(DeleteItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
            }));
        }
    };
    let hook_type = config.name.to_lowercase();

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = serde_json::json!({
        "item_type": &item_type,
        "item_id": &req.item_id,
        "force": req.force,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        &hook_type,
        HookOperation::Delete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DeleteItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
        }));
    }

    match generic_delete(project_path, &item_type, &config, &req.item_id, req.force).await {
        Ok(()) => {
            maybe_run_post_hooks(
                project_path,
                &hook_type,
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(DeleteItemResponse {
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                &hook_type,
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(DeleteItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
            }))
        }
    }
}
