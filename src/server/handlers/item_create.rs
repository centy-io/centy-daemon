use std::collections::HashMap;
use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_create;
use crate::item::generic::types::CreateGenericItemOptions;
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::helpers::{nonempty, nonzero_u32};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{CreateItemRequest, CreateItemResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_type_resolve::{
    normalize_item_type, resolve_hook_item_type, resolve_item_type_config,
};

pub async fn create_item(req: CreateItemRequest) -> Result<Response<CreateItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let item_type = normalize_item_type(&req.item_type);

    // Resolve config
    let config = match resolve_item_type_config(project_path, &item_type).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Response::new(CreateItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }));
        }
    };

    // Pre-hook
    let hook_item_type = resolve_hook_item_type(&item_type);
    let hook_project_path = req.project_path.clone();
    let hook_request_data = serde_json::json!({
        "item_type": &item_type,
        "title": &req.title,
        "body": &req.body,
        "priority": req.priority,
        "status": &req.status,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        hook_item_type,
        HookOperation::Create,
        &hook_project_path,
        None,
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(CreateItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    // Convert custom_fields from map<string,string> to map<string, serde_json::Value>
    let custom_fields: HashMap<String, serde_json::Value> = req
        .custom_fields
        .into_iter()
        .map(|(k, v)| {
            let val = serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v));
            (k, val)
        })
        .collect();

    let options = CreateGenericItemOptions {
        title: req.title,
        body: req.body,
        id: None,
        status: nonempty(req.status),
        priority: nonzero_u32(req.priority),
        custom_fields,
    };

    match generic_create(project_path, &config, options).await {
        Ok(item) => {
            maybe_run_post_hooks(
                project_path,
                hook_item_type,
                HookOperation::Create,
                &hook_project_path,
                Some(&item.id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(CreateItemResponse {
                success: true,
                error: String::new(),
                item: Some(generic_item_to_proto(&item)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                hook_item_type,
                HookOperation::Create,
                &hook_project_path,
                None,
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(CreateItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                item: None,
            }))
        }
    }
}
