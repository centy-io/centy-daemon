use std::collections::HashMap;
use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_update;
use crate::item::generic::types::UpdateGenericItemOptions;
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::helpers::{nonempty, nonzero_u32};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{UpdateItemRequest, UpdateItemResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn update_item(req: UpdateItemRequest) -> Result<Response<UpdateItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    // Resolve config
    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(UpdateItemResponse {
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
    let hook_data = serde_json::json!({
        "item_type": &item_type,
        "item_id": &req.item_id,
        "title": &req.title,
        "body": &req.body,
        "priority": req.priority,
        "status": &req.status,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        &hook_type,
        HookOperation::Update,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_data.clone()),
    )
    .await
    {
        return Ok(Response::new(UpdateItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    // Convert custom_fields
    let custom_fields: HashMap<String, serde_json::Value> = req
        .custom_fields
        .into_iter()
        .map(|(k, v)| {
            let val = serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v));
            (k, val)
        })
        .collect();

    let options = UpdateGenericItemOptions {
        title: nonempty(req.title),
        body: nonempty(req.body),
        status: nonempty(req.status),
        priority: nonzero_u32(req.priority),
        custom_fields,
    };

    match generic_update(project_path, &config, &req.item_id, options).await {
        Ok(item) => {
            maybe_run_post_hooks(
                project_path,
                &hook_type,
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_data),
                true,
            )
            .await;
            Ok(Response::new(UpdateItemResponse {
                success: true,
                error: String::new(),
                item: Some(generic_item_to_proto(&item)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                &hook_type,
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_data),
                false,
            )
            .await;
            Ok(Response::new(UpdateItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                item: None,
            }))
        }
    }
}
