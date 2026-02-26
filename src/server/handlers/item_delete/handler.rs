use std::path::Path;
use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{DeleteItemRequest, DeleteItemResponse};
use crate::server::structured_error::to_error_json;
use crate::user::delete_user;
use tonic::{Response, Status};
use super::super::item_type_resolve::{resolve_item_id, resolve_item_type_config};
use super::operation::do_delete;
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn delete_item(req: DeleteItemRequest) -> Result<Response<DeleteItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(DeleteItemResponse {
            success: false, error: to_error_json(&req.project_path, &e),
        }));
    }
    let lower = req.item_type.to_lowercase();
    if lower == "user" || lower == "users" {
        return match delete_user(project_path, &req.item_id).await {
            Ok(_) => Ok(Response::new(DeleteItemResponse { success: true, error: String::new() })),
            Err(e) => Ok(Response::new(DeleteItemResponse {
                success: false, error: to_error_json(&req.project_path, &e),
            })),
        };
    }
    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => return Ok(Response::new(DeleteItemResponse {
            success: false, error: to_error_json(&req.project_path, &e),
        })),
    };
    let hook_type = config.name.to_lowercase();
    let item_id = match resolve_item_id(project_path, &item_type, &config, &req.item_id).await {
        Ok(id) => id,
        Err(e) => return Ok(Response::new(DeleteItemResponse {
            success: false, error: to_error_json(&req.project_path, &e),
        })),
    };
    let hook_project_path = req.project_path.clone();
    let hook_item_id = item_id.clone();
    let hook_data = serde_json::json!({"item_type": &item_type, "item_id": &item_id, "force": req.force});
    if let Err(e) = maybe_run_pre_hooks(
        project_path, &hook_type, HookOperation::Delete,
        &hook_project_path, Some(&hook_item_id), Some(hook_data.clone()),
    ).await {
        return Ok(Response::new(DeleteItemResponse {
            success: false, error: to_error_json(&req.project_path, &e),
        }));
    }
    Ok(Response::new(do_delete(
        project_path, &item_type, &config, &item_id, req.force, &hook_type,
        &hook_project_path, &hook_item_id, hook_data, &req.project_path,
    ).await))
}
