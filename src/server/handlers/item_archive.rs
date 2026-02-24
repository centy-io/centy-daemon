use std::collections::HashMap;
use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::generic::storage::{generic_move, generic_update};
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{ArchiveItemRequest, ArchiveItemResponse};
use crate::server::assert_service::assert_initialized;
use crate::server::structured_error::to_error_json;
use mdstore::{TypeConfig, UpdateOptions};
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

/// The folder name used for archived items.
pub const ARCHIVED_FOLDER: &str = "archived";

/// After a successful move, stamp `original_item_type` on the archived item
/// and return the appropriate `ArchiveItemResponse`.
async fn set_original_item_type_and_respond(
    project_path: &Path,
    project_path_str: &str,
    archived_type: &str,
    archived_config: &TypeConfig,
    source_type: &str,
    moved_item: mdstore::Item,
) -> Result<Response<ArchiveItemResponse>, Status> {
    let mut custom_fields = HashMap::new();
    custom_fields.insert(
        "original_item_type".to_string(),
        serde_json::Value::String(source_type.to_string()),
    );
    let update_opts = UpdateOptions {
        custom_fields,
        ..Default::default()
    };

    match generic_update(
        project_path,
        archived_type,
        archived_config,
        &moved_item.id,
        update_opts,
    )
    .await
    {
        Ok(updated_item) => Ok(Response::new(ArchiveItemResponse {
            success: true,
            error: String::new(),
            item: Some(generic_item_to_proto(&updated_item, archived_type)),
        })),
        Err(e) => Ok(Response::new(ArchiveItemResponse {
            success: false,
            error: to_error_json(project_path_str, &e),
            item: Some(generic_item_to_proto(&moved_item, archived_type)),
        })),
    }
}

/// Archive an item by moving it to the `archived/` folder and recording
/// its original item type in the `original_item_type` custom field.
pub async fn archive_item(
    req: ArchiveItemRequest,
) -> Result<Response<ArchiveItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(ArchiveItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            item: None,
        }));
    }

    // Resolve source config
    let (source_type, source_config) =
        match resolve_item_type_config(project_path, &req.item_type).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(ArchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    item: None,
                }));
            }
        };

    // Resolve archived config
    let (archived_type, archived_config) =
        match resolve_item_type_config(project_path, ARCHIVED_FOLDER).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(ArchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    item: None,
                }));
            }
        };

    let hook_type = source_config.name.to_lowercase();
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = serde_json::json!({
        "item_type": &req.item_type,
        "item_id": &req.item_id,
        "target_folder": ARCHIVED_FOLDER,
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
        return Ok(Response::new(ArchiveItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            item: None,
        }));
    }

    // Move item to archived folder
    let move_result = generic_move(
        project_path,
        project_path,
        &source_type,
        &archived_type,
        &source_config,
        &archived_config,
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
        Ok(result) => {
            // Set original_item_type custom field to the source folder name
            set_original_item_type_and_respond(
                project_path,
                &req.project_path,
                &archived_type,
                &archived_config,
                &source_type,
                result.item,
            )
            .await
        }
        Err(e) => Ok(Response::new(ArchiveItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            item: None,
        })),
    }
}
