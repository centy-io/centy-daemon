use super::super::item_type_resolve::{resolve_item_id, resolve_item_type_config};
use super::operation::do_restore;
use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{RestoreItemRequest, RestoreItemResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn restore_item(
    req: RestoreItemRequest,
) -> Result<Response<RestoreItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(RestoreItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }
    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(RestoreItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };
    let hook_type = config.name.to_lowercase();
    let item_id = match resolve_item_id(project_path, &item_type, &config, &req.item_id).await {
        Ok(id) => id,
        Err(e) => {
            return Ok(Response::new(RestoreItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };
    let hook_project_path = req.project_path.clone();
    let hook_item_id = item_id.clone();
    let hook_data = serde_json::json!({"item_type": &item_type, "item_id": &item_id});
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        &hook_type,
        HookOperation::Restore,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_data.clone()),
    )
    .await
    {
        return Ok(Response::new(RestoreItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }
    Ok(Response::new(
        do_restore(
            project_path,
            &item_type,
            &item_id,
            &hook_type,
            &hook_project_path,
            &hook_item_id,
            hook_data,
            &req.project_path,
        )
        .await,
    ))
}
