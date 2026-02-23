use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::item::generic::storage::{generic_get, generic_move};
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{UnarchiveItemRequest, UnarchiveItemResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_archive::ARCHIVED_FOLDER;
use super::item_type_resolve::resolve_item_type_config;

/// Unarchive an item by moving it from the `archived/` folder back to its
/// original folder (determined by the `original_item_type` custom field).
pub async fn unarchive_item(
    req: UnarchiveItemRequest,
) -> Result<Response<UnarchiveItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Resolve archived config
    let (archived_type, archived_config) =
        match resolve_item_type_config(project_path, ARCHIVED_FOLDER).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(UnarchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    ..Default::default()
                }));
            }
        };

    // Get the archived item to read its original_item_type
    let archived_item = match generic_get(project_path, &archived_type, &req.item_id).await {
        Ok(item) => item,
        Err(e) => {
            return Ok(Response::new(UnarchiveItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }));
        }
    };

    // Determine the target folder
    let target_folder = if req.target_item_type.is_empty() {
        // Read from original_item_type custom field
        match archived_item
            .frontmatter
            .custom_fields
            .get("original_item_type")
        {
            Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
            _ => {
                let err = ItemError::custom(
                    "original_item_type not set on archived item; \
                     provide target_item_type to override",
                );
                return Ok(Response::new(UnarchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &err),
                    ..Default::default()
                }));
            }
        }
    } else {
        req.target_item_type.clone()
    };

    // Resolve target config
    let (target_type, target_config) =
        match resolve_item_type_config(project_path, &target_folder).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(UnarchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    ..Default::default()
                }));
            }
        };

    let hook_type = archived_config.name.to_lowercase();
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = serde_json::json!({
        "item_type": ARCHIVED_FOLDER,
        "item_id": &req.item_id,
        "target_folder": &target_folder,
    });

    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        &hook_type,
        HookOperation::Move,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(UnarchiveItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    // Move item back to original folder
    let move_result = generic_move(
        project_path,
        project_path,
        &archived_type,
        &target_type,
        &archived_config,
        &target_config,
        &req.item_id,
        None,
    )
    .await;

    let success = move_result.is_ok();
    maybe_run_post_hooks(
        project_path,
        &hook_type,
        HookOperation::Move,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data),
        success,
    )
    .await;

    match move_result {
        Ok(result) => Ok(Response::new(UnarchiveItemResponse {
            success: true,
            error: String::new(),
            item: Some(generic_item_to_proto(&result.item, &target_type)),
            original_item_type: target_folder,
        })),
        Err(e) => Ok(Response::new(UnarchiveItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        })),
    }
}
