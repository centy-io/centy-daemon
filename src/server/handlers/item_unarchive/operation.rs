#![allow(unknown_lints, max_lines_per_file)]
use super::super::item_archive::ARCHIVED_FOLDER;
use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::item::generic::storage::generic_move;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::UnarchiveItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::{Item, TypeConfig};
use std::path::Path;
use tonic::{Response, Status};
/// Determine the destination folder for an unarchive operation.
pub(super) fn resolve_target_folder(
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
/// Execute the move from `archived/` to `target_type`, run surrounding hooks.
pub(super) async fn move_and_respond(
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
        "item_type": ARCHIVED_FOLDER, "item_id": item_id,
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
