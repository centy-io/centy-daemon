use super::super::item_type_resolve::{resolve_item_id, resolve_item_type_config};
use super::operation::do_delete;
use crate::config::item_type_config::ItemTypeRegistry;
use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{DeleteItemRequest, DeleteItemResponse};
use crate::server::structured_error::to_error_json;
use crate::user::delete_user;
use std::path::Path;
use tonic::{Response, Status};
fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + crate::server::error_mapping::ToStructuredError),
) -> Response<DeleteItemResponse> {
    Response::new(DeleteItemResponse {
        success: false,
        error: to_error_json(cwd, e),
    })
}
pub async fn delete_item(req: DeleteItemRequest) -> Result<Response<DeleteItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let lower = req.item_type.to_lowercase();
    if lower == "user" || lower == "users" {
        return match delete_user(project_path, &req.item_id).await {
            Ok(_) => Ok(Response::new(DeleteItemResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(err_resp(&req.project_path, &e)),
        };
    }
    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => return Ok(err_resp(&req.project_path, &e)),
    };
    let hook_type = config.name.to_lowercase();
    // If the display-number lookup fails because the item is absent from the
    // project, fall through with the original identifier so that `do_delete`
    // can attempt the org-repo fallback.
    let item_id = match resolve_item_id(project_path, &item_type, &config, &req.item_id).await {
        Ok(id) => id,
        Err(ItemError::NotFound(_)) => req.item_id.clone(),
        Err(e) => return Ok(err_resp(&req.project_path, &e)),
    };
    // When the item type has soft-delete disabled, always hard-delete regardless
    // of the client's force flag.
    let soft_delete_enabled = ItemTypeRegistry::build(project_path)
        .await
        .ok()
        .and_then(|r| {
            r.resolve(&req.item_type)
                .map(|(_, c)| c.features.soft_delete)
        })
        .unwrap_or(true);
    let force = req.force || !soft_delete_enabled;
    let hook_project_path = req.project_path.clone();
    let hook_item_id = item_id.clone();
    let hook_data =
        serde_json::json!({"item_type": &item_type, "item_id": &item_id, "force": force});
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        &hook_type,
        HookOperation::Delete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
    }
    Ok(Response::new(
        do_delete(
            project_path,
            &item_type,
            &config,
            &item_id,
            force,
            &hook_type,
            &hook_project_path,
            &hook_item_id,
            hook_data,
            &req.project_path,
        )
        .await,
    ))
}
