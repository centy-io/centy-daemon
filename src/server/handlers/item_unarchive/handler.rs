use super::super::item_archive::ARCHIVED_FOLDER;
use super::super::item_type_resolve::resolve_item_type_config;
use super::operation::{move_and_respond, resolve_target_folder};
use crate::item::generic::storage::generic_get;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::proto::{UnarchiveItemRequest, UnarchiveItemResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};
/// Unarchive an item by moving it from `archived/` back to its original folder.
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
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
    let (archived_type, archived_config) =
        match resolve_item_type_config(project_path, ARCHIVED_FOLDER).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(UnarchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    ..Default::default()
                }))
            }
        };
    let archived_item = match generic_get(project_path, &archived_type, &req.item_id).await {
        Ok(item) => item,
        Err(e) => {
            return Ok(Response::new(UnarchiveItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };
    let target_folder =
        match resolve_target_folder(&req.project_path, &archived_item, &req.target_item_type) {
            Ok(folder) => folder,
            Err(error_json) => {
                return Ok(Response::new(UnarchiveItemResponse {
                    success: false,
                    error: error_json,
                    ..Default::default()
                }))
            }
        };
    let (target_type, target_config) =
        match resolve_item_type_config(project_path, &target_folder).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(UnarchiveItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    ..Default::default()
                }))
            }
        };
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
