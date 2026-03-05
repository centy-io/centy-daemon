use super::archive_move::run_archive_hooks_and_move;
use super::operation::{err_resp, resolve_both_types, ARCHIVED_FOLDER};
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::proto::{ArchiveItemRequest, ArchiveItemResponse};
use std::path::Path;
use tonic::{Response, Status};
/// Archive an item by moving it to the `archived/` folder and recording
/// its original item type in the `original_item_type` custom field.
pub async fn archive_item(
    req: ArchiveItemRequest,
) -> Result<Response<ArchiveItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(err_resp(&req.project_path, &e)));
    }
    let ((source_type, source_config), (archived_type, archived_config)) =
        match resolve_both_types(project_path, &req.item_type).await {
            Ok(pair) => pair,
            Err(e) => return Ok(Response::new(err_resp(&req.project_path, &e))),
        };
    let hook_type = source_config.name.to_lowercase();
    let hook_request_data = serde_json::json!({
        "item_type": &req.item_type, "item_id": &req.item_id,
        "target_folder": ARCHIVED_FOLDER,
    });
    run_archive_hooks_and_move(
        project_path,
        &req.project_path,
        &source_type,
        &archived_type,
        &source_config,
        &archived_config,
        &hook_type,
        &req.item_id,
        hook_request_data,
    )
    .await
}
