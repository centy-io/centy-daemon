use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::generic::storage::{generic_get, generic_restore};
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{RestoreItemRequest, RestoreItemResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn restore_item(
    req: RestoreItemRequest,
) -> Result<Response<RestoreItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    // Resolve config
    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(RestoreItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }));
        }
    };
    let hook_type = config.name.to_lowercase();

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = serde_json::json!({
        "item_type": &item_type,
        "item_id": &req.item_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        &hook_type,
        HookOperation::Restore,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(RestoreItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    match generic_restore(project_path, &item_type, &req.item_id).await {
        Ok(()) => {
            maybe_run_post_hooks(
                project_path,
                &hook_type,
                HookOperation::Restore,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            // Fetch the item after restore to return it
            let item = generic_get(project_path, &item_type, &req.item_id)
                .await
                .ok()
                .map(|i| generic_item_to_proto(&i, &item_type));

            Ok(Response::new(RestoreItemResponse {
                success: true,
                error: String::new(),
                item,
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                &hook_type,
                HookOperation::Restore,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(RestoreItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                item: None,
            }))
        }
    }
}
