use crate::hooks::HookOperation;
use crate::server::assert_service::assert_initialized;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::MoveItemRequest;
use crate::server::proto::MoveItemResponse;
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::Response;
pub(super) struct MoveHookContext {
    pub(super) hook_type: String,
    pub(super) hook_project_path: String,
    pub(super) hook_item_id: String,
    pub(super) hook_request_data: serde_json::Value,
}
/// Build hook context and run pre-hooks for a move operation.
pub(super) async fn prepare_move_hooks(
    req: &MoveItemRequest,
    source_path: &Path,
    source_config_name: &str,
) -> Result<MoveHookContext, Response<MoveItemResponse>> {
    let hook_type = source_config_name.to_lowercase();
    let hook_project_path = req.source_project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = serde_json::json!({
        "item_type": &req.item_type, "item_id": &req.item_id,
        "source_project_path": &req.source_project_path,
        "target_project_path": &req.target_project_path,
        "new_id": &req.new_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        source_path,
        &hook_type,
        HookOperation::Move,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
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
        hook_type,
        hook_project_path,
        hook_item_id,
        hook_request_data,
    })
}
/// Assert both source and target projects are initialized.
pub(super) async fn assert_both_initialized(
    req: &MoveItemRequest,
    source_path: &Path,
    target_path: &Path,
) -> Result<(), Response<MoveItemResponse>> {
    if let Err(e) = assert_initialized(source_path).await {
        return Err(Response::new(MoveItemResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }
    if let Err(e) = assert_initialized(target_path).await {
        return Err(Response::new(MoveItemResponse {
            success: false,
            error: to_error_json(&req.target_project_path, &e),
            ..Default::default()
        }));
    }
    Ok(())
}
