use crate::item::generic::storage::{generic_move, generic_update};
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::proto::ArchiveItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::{TypeConfig, UpdateOptions};
use std::collections::HashMap;
use std::path::Path;
use tonic::{Response, Status};
/// The folder name used for archived items.
pub const ARCHIVED_FOLDER: &str = "archived";
/// After a successful move, stamp `original_item_type` on the archived item
/// and return the appropriate `ArchiveItemResponse`.
pub(super) async fn set_original_item_type_and_respond(
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
pub(super) async fn do_move_to_archive(
    project_path: &Path,
    source_type: &str,
    archived_type: &str,
    source_config: &TypeConfig,
    archived_config: &TypeConfig,
    item_id: &str,
) -> Result<mdstore::MoveResult, crate::item::core::error::ItemError> {
    generic_move(
        project_path,
        project_path,
        source_type,
        archived_type,
        source_config,
        archived_config,
        item_id,
        None,
    )
    .await
}
