use super::super::item_archive::ARCHIVED_FOLDER;
use super::super::item_type_resolve::resolve_item_type_config;
use super::helpers::resolve_target_folder;
use super::operation::{err_resp, err_resp_str, move_and_respond};
use crate::item::generic::storage::generic_get;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::proto::{UnarchiveItemRequest, UnarchiveItemResponse};
use std::path::Path;
use tonic::{Response, Status};
/// Unarchive an item by moving it from `archived/` back to its original folder.
pub async fn unarchive_item(
    req: UnarchiveItemRequest,
) -> Result<Response<UnarchiveItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(Response::new(err_resp(&req.project_path, &e)));
    }
    let (archived_type, archived_config) =
        match resolve_item_type_config(project_path, ARCHIVED_FOLDER).await {
            Ok(pair) => pair,
            Err(e) => return Ok(Response::new(err_resp(&req.project_path, &e))),
        };
    let archived_item = match generic_get(project_path, &archived_type, &req.item_id).await {
        Ok(item) => item,
        Err(e) => return Ok(Response::new(err_resp(&req.project_path, &e))),
    };
    let target_folder =
        match resolve_target_folder(&req.project_path, &archived_item, &req.target_item_type) {
            Ok(folder) => folder,
            Err(error_json) => return Ok(Response::new(err_resp_str(error_json))),
        };
    let (target_type, target_config) =
        match resolve_item_type_config(project_path, &target_folder).await {
            Ok(pair) => pair,
            Err(e) => return Ok(Response::new(err_resp(&req.project_path, &e))),
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
