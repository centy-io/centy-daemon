#![allow(unknown_lints, max_lines_per_file)]
use super::super::item_type_resolve::resolve_item_type_config;
use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::manifest;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::MoveItemRequest;
use crate::server::proto::MoveItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::TypeConfig;
use std::path::Path;
use tonic::{Response, Status};
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
/// Build a successful `MoveItemResponse` from a move/rename result.
pub(super) async fn build_ok_response(
    result: mdstore::MoveResult,
    source_path: &Path,
    target_path: &Path,
    target_type: &str,
) -> MoveItemResponse {
    let source_manifest = manifest::read_manifest(source_path).await.ok().flatten();
    let target_manifest = manifest::read_manifest(target_path).await.ok().flatten();
    MoveItemResponse {
        success: true,
        error: String::new(),
        item: Some(generic_item_to_proto(&result.item, target_type)),
        old_id: result.old_id,
        source_manifest: source_manifest.map(|m| manifest_to_proto(&m)),
        target_manifest: target_manifest.map(|m| manifest_to_proto(&m)),
    }
}
/// Run post-hooks and return the move response.
pub(super) async fn finish_move(
    move_result: Result<mdstore::MoveResult, ItemError>,
    source_path: &Path,
    target_path: &Path,
    target_type: &str,
    hook_type: &str,
    hook_project_path: &str,
    hook_item_id: &str,
    hook_request_data: serde_json::Value,
    error_project_path: &str,
) -> Result<Response<MoveItemResponse>, Status> {
    let success = move_result.is_ok();
    maybe_run_post_hooks(
        source_path,
        hook_type,
        HookOperation::Move,
        hook_project_path,
        Some(hook_item_id),
        Some(hook_request_data),
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
/// Resolve item type configs for source and target paths.
pub(super) async fn resolve_configs(
    req: &MoveItemRequest,
    source_path: &Path,
    target_path: &Path,
) -> Result<((String, TypeConfig), (String, TypeConfig)), Response<MoveItemResponse>> {
    let source = resolve_item_type_config(source_path, &req.item_type)
        .await
        .map_err(|e| {
            Response::new(MoveItemResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                ..Default::default()
            })
        })?;
    let target = resolve_item_type_config(target_path, &req.item_type)
        .await
        .map_err(|e| {
            Response::new(MoveItemResponse {
                success: false,
                error: to_error_json(&req.target_project_path, &e),
                ..Default::default()
            })
        })?;
    Ok((source, target))
}
