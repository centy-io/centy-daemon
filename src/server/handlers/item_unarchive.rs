use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::item::generic::storage::{generic_get, generic_move};
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{UnarchiveItemRequest, UnarchiveItemResponse};
use crate::server::assert_service::assert_initialized;
use crate::server::structured_error::to_error_json;
use mdstore::{Item, TypeConfig};
use tonic::{Response, Status};

use super::item_archive::ARCHIVED_FOLDER;
use super::item_type_resolve::resolve_item_type_config;

/// Determine the destination folder for an unarchive operation.
///
/// If `requested` is non-empty, it is used directly. Otherwise the
/// `original_item_type` custom field on `archived_item` is consulted.
/// Returns an error string (pre-serialized JSON) when the folder cannot be
/// determined.
fn resolve_target_folder(
    project_path_str: &str,
    archived_item: &Item,
    requested: &str,
) -> Result<String, String> {
    if !requested.is_empty() {
        return Ok(requested.to_string());
    }

    match archived_item
        .frontmatter
        .custom_fields
        .get("original_item_type")
    {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        _ => {
            let err = ItemError::custom(
                "original_item_type not set on archived item; \
                 provide target_item_type to override",
            );
            Err(to_error_json(project_path_str, &err))
        }
    }
}

/// Execute the move from `archived/` to `target_type`, run surrounding hooks,
/// and return the final `UnarchiveItemResponse`.
async fn move_and_respond(
    project_path: &Path,
    project_path_str: &str,
    archived_type: &str,
    archived_config: &TypeConfig,
    target_type: &str,
    target_config: &TypeConfig,
    item_id: &str,
    target_folder: String,
) -> Result<Response<UnarchiveItemResponse>, Status> {
    let hook_type = archived_config.name.to_lowercase();
    let hook_request_data = serde_json::json!({
        "item_type": ARCHIVED_FOLDER,
        "item_id": item_id,
        "target_folder": &target_folder,
    });

    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        &hook_type,
        HookOperation::Move,
        project_path_str,
        Some(item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(UnarchiveItemResponse {
            success: false,
            error: to_error_json(project_path_str, &e),
            ..Default::default()
        }));
    }

    let move_result = generic_move(
        project_path,
        project_path,
        archived_type,
        target_type,
        archived_config,
        target_config,
        item_id,
        None,
    )
    .await;

    let success = move_result.is_ok();
    maybe_run_post_hooks(
        project_path,
        &hook_type,
        HookOperation::Move,
        project_path_str,
        Some(item_id),
        Some(hook_request_data),
        success,
    )
    .await;

    match move_result {
        Ok(result) => Ok(Response::new(UnarchiveItemResponse {
            success: true,
            error: String::new(),
            item: Some(generic_item_to_proto(&result.item, target_type)),
            original_item_type: target_folder,
        })),
        Err(e) => Ok(Response::new(UnarchiveItemResponse {
            success: false,
            error: to_error_json(project_path_str, &e),
            ..Default::default()
        })),
    }
}

/// Unarchive an item by moving it from the `archived/` folder back to its
/// original folder (determined by the `original_item_type` custom field).
pub async fn unarchive_item(
    req: UnarchiveItemRequest,
) -> Result<Response<UnarchiveItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(UnarchiveItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

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
    let target_folder =
        match resolve_target_folder(&req.project_path, &archived_item, &req.target_item_type) {
            Ok(folder) => folder,
            Err(error_json) => {
                return Ok(Response::new(UnarchiveItemResponse {
                    success: false,
                    error: error_json,
                    ..Default::default()
                }));
            }
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

    // Move item back to original folder (with surrounding hooks)
    move_and_respond(
        project_path,
        &req.project_path,
        &archived_type,
        &archived_config,
        &target_type,
        &target_config,
        &req.item_id,
        target_folder,
    )
    .await
}
