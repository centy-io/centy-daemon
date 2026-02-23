use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::item::generic::storage::{generic_move, generic_rename_slug};
use crate::manifest;
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{MoveItemRequest, MoveItemResponse};
use crate::server::structured_error::to_error_json;
use mdstore::{IdStrategy, TypeConfig};
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

/// Build a successful `MoveItemResponse` from a move/rename result.
async fn build_ok_response(
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
#[allow(clippy::too_many_arguments)]
async fn finish_move(
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

/// Resolve item type configs for source and target paths.
async fn resolve_configs(
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

pub async fn move_item(req: MoveItemRequest) -> Result<Response<MoveItemResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());

    let source_path = Path::new(&req.source_project_path);
    let target_path = Path::new(&req.target_project_path);

    let ((source_type, source_config), (target_type, target_config)) =
        match resolve_configs(&req, source_path, target_path).await {
            Ok(pair) => pair,
            Err(resp) => return Ok(resp),
        };

    let hook_type = source_config.name.to_lowercase();
    let hook_project_path = req.source_project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = serde_json::json!({
        "item_type": &req.item_type,
        "item_id": &req.item_id,
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

    // Same-project slug rename: mdstore::move_item rejects same-directory moves.
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
