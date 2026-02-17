use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_move;
use crate::manifest;
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{MoveItemRequest, MoveItemResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn move_item(req: MoveItemRequest) -> Result<Response<MoveItemResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());

    let source_path = Path::new(&req.source_project_path);
    let target_path = Path::new(&req.target_project_path);

    // Resolve source config
    let (source_type, source_config) =
        match resolve_item_type_config(source_path, &req.item_type).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(MoveItemResponse {
                    success: false,
                    error: to_error_json(&req.source_project_path, &e),
                    ..Default::default()
                }));
            }
        };

    // Resolve target config (same item type in target project)
    let (target_type, target_config) =
        match resolve_item_type_config(target_path, &req.item_type).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(MoveItemResponse {
                    success: false,
                    error: to_error_json(&req.target_project_path, &e),
                    ..Default::default()
                }));
            }
        };

    let hook_type = source_config.name.to_lowercase();

    // Pre-hook
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

    let new_id = if req.new_id.is_empty() {
        None
    } else {
        Some(req.new_id.as_str())
    };

    match generic_move(
        source_path,
        target_path,
        &source_type,
        &target_type,
        &source_config,
        &target_config,
        &req.item_id,
        new_id,
    )
    .await
    {
        Ok(result) => {
            maybe_run_post_hooks(
                source_path,
                &hook_type,
                HookOperation::Move,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            // Read both manifests for the response
            let source_manifest = manifest::read_manifest(source_path).await.ok().flatten();
            let target_manifest = manifest::read_manifest(target_path).await.ok().flatten();

            Ok(Response::new(MoveItemResponse {
                success: true,
                error: String::new(),
                item: Some(generic_item_to_proto(&result.item, &target_type)),
                old_id: result.old_id,
                source_manifest: source_manifest.map(|m| manifest_to_proto(&m)),
                target_manifest: target_manifest.map(|m| manifest_to_proto(&m)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                source_path,
                &hook_type,
                HookOperation::Move,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(MoveItemResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                ..Default::default()
            }))
        }
    }
}
