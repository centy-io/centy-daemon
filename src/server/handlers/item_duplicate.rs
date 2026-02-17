use std::path::{Path, PathBuf};

use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_duplicate;
use crate::item::generic::types::DuplicateGenericItemOptions;
use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::nonempty;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DuplicateItemRequest, DuplicateItemResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn duplicate_item(
    req: DuplicateItemRequest,
) -> Result<Response<DuplicateItemResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());

    let target_project_path = Path::new(&req.target_project_path);

    // Resolve config from target project
    let (item_type, config) =
        match resolve_item_type_config(target_project_path, &req.item_type).await {
            Ok(pair) => pair,
            Err(e) => {
                return Ok(Response::new(DuplicateItemResponse {
                    success: false,
                    error: to_error_json(&req.source_project_path, &e),
                    ..Default::default()
                }));
            }
        };
    let hook_type = config.name.to_lowercase();

    // Check if duplicate feature is enabled
    if !config.features.duplicate {
        return Ok(Response::new(DuplicateItemResponse {
            success: false,
            error: to_error_json(
                &req.source_project_path,
                &crate::item::core::error::ItemError::FeatureNotEnabled("duplicate".to_string()),
            ),
            ..Default::default()
        }));
    }

    // Pre-hook
    let hook_project_path = req.source_project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = serde_json::json!({
        "item_type": &req.item_type,
        "source_project_path": &req.source_project_path,
        "target_project_path": &req.target_project_path,
        "item_id": &req.item_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        Path::new(&hook_project_path),
        &hook_type,
        HookOperation::Duplicate,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DuplicateItemResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }

    let options = DuplicateGenericItemOptions {
        source_project_path: PathBuf::from(&req.source_project_path),
        target_project_path: PathBuf::from(&req.target_project_path),
        item_id: req.item_id,
        new_id: nonempty(req.new_id),
        new_title: nonempty(req.new_title),
    };

    match generic_duplicate(&item_type, &config, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                &hook_type,
                HookOperation::Duplicate,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            // Read updated manifest from target project
            let manifest = read_manifest(target_project_path).await.ok().flatten();

            Ok(Response::new(DuplicateItemResponse {
                success: true,
                error: String::new(),
                item: Some(generic_item_to_proto(&result.item, &item_type)),
                original_id: result.original_id,
                manifest: manifest.as_ref().map(manifest_to_proto),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                &hook_type,
                HookOperation::Duplicate,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(DuplicateItemResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                item: None,
                original_id: String::new(),
                manifest: None,
            }))
        }
    }
}
