#![allow(unknown_lints, max_lines_per_file)]
use super::helpers::{assert_both_initialized, finish_move, resolve_configs};
use crate::hooks::HookOperation;
use crate::item::generic::storage::{generic_move, generic_rename_slug};
use crate::registry::track_project_async;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{MoveItemRequest, MoveItemResponse};
use crate::server::structured_error::to_error_json;
use mdstore::IdStrategy;
use std::path::Path;
use tonic::{Response, Status};
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn move_item(req: MoveItemRequest) -> Result<Response<MoveItemResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());
    let source_path = Path::new(&req.source_project_path);
    let target_path = Path::new(&req.target_project_path);
    if let Err(resp) = assert_both_initialized(&req, source_path, target_path).await {
        return Ok(resp);
    }
    let ((source_type, source_config), (target_type, target_config)) =
        match resolve_configs(&req, source_path, target_path).await {
            Ok(pair) => pair,
            Err(resp) => return Ok(resp),
        };
    let hook_type = source_config.name.to_lowercase();
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
        return Ok(Response::new(MoveItemResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }
    if req.source_project_path == req.target_project_path
        && !req.new_id.is_empty()
        && source_config.identifier == IdStrategy::Slug
    {
        let result = generic_rename_slug(
            source_path,
            &source_type,
            &source_config,
            &req.item_id,
            &req.new_id,
        )
        .await;
        return finish_move(
            result,
            source_path,
            target_path,
            &target_type,
            &hook_type,
            &hook_project_path,
            &hook_item_id,
            hook_request_data,
            &req.source_project_path,
        )
        .await;
    }
    let new_id = if req.new_id.is_empty() {
        None
    } else {
        Some(req.new_id.as_str())
    };
    let result = generic_move(
        source_path,
        target_path,
        &source_type,
        &target_type,
        &source_config,
        &target_config,
        &req.item_id,
        new_id,
    )
    .await;
    finish_move(
        result,
        source_path,
        target_path,
        &target_type,
        &hook_type,
        &hook_project_path,
        &hook_item_id,
        hook_request_data,
        &req.source_project_path,
    )
    .await
}
