use super::helpers::build_ok_response;
use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{MoveItemRequest, MoveItemResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

pub(super) struct MoveHookContext {
    pub(super) item_type: String,
    pub(super) project_path: String,
    pub(super) item_id: String,
    pub(super) request_data: serde_json::Value,
}
/// Build hook context and run pre-hooks for a move operation.
pub(super) async fn prepare_move_hooks(
    req: &MoveItemRequest,
    source_path: &Path,
    source_config_name: &str,
) -> Result<MoveHookContext, Response<MoveItemResponse>> {
    let hook_type = source_config_name.to_lowercase();
    let project_path = req.source_project_path.clone();
    let item_id = req.item_id.clone();
    let request_data = serde_json::json!({
        "item_type": &req.item_type, "item_id": &req.item_id,
        "source_project_path": &req.source_project_path,
        "target_project_path": &req.target_project_path,
        "new_id": &req.new_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        source_path,
        &hook_type,
        HookOperation::Move,
        &project_path,
        Some(&item_id),
        Some(request_data.clone()),
    )
    .await
    {
        return Err(Response::new(MoveItemResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }
    Ok(MoveHookContext {
        item_type: hook_type,
        project_path,
        item_id,
        request_data,
    })
}
/// Run post-hooks and return the move response.
pub(super) async fn finish_move(
    move_result: Result<mdstore::MoveResult, ItemError>,
    source_path: &Path,
    target_path: &Path,
    target_type: &str,
    hook_type: &str,
    project_path: &str,
    item_id: &str,
    request_data: serde_json::Value,
    error_project_path: &str,
) -> Result<Response<MoveItemResponse>, Status> {
    let success = move_result.is_ok();
    maybe_run_post_hooks(
        source_path,
        hook_type,
        HookOperation::Move,
        project_path,
        Some(item_id),
        Some(request_data),
        success,
    )
    .await;
    match move_result {
        Ok(result) => Ok(Response::new(
            build_ok_response(result, source_path, target_path, target_type).await,
        )),
        Err(e) => Ok(Response::new(MoveItemResponse {
            success: false,
            error: to_error_json(error_project_path, &e),
            ..Default::default()
        })),
    }
}
