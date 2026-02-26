use super::super::item_type_resolve::resolve_item_type_config;
use super::operation::{do_move_to_archive, set_original_item_type_and_respond, ARCHIVED_FOLDER};
use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{ArchiveItemRequest, ArchiveItemResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};
/// Archive an item by moving it to the `archived/` folder and recording
/// its original item type in the `original_item_type` custom field.
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
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
    let (source_type, source_config) =
        match resolve_item_type_config(project_path, &req.item_type).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(ArchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    item: None,
                }))
            }
        };
    let (archived_type, archived_config) =
        match resolve_item_type_config(project_path, ARCHIVED_FOLDER).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(ArchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    item: None,
                }))
            }
        };
    let hook_type = source_config.name.to_lowercase();
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = serde_json::json!({
        "item_type": &req.item_type, "item_id": &req.item_id,
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
    let move_result = do_move_to_archive(
        project_path,
        &source_type,
        &archived_type,
        &source_config,
        &archived_config,
        &req.item_id,
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
