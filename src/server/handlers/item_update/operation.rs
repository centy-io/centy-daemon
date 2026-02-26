use std::collections::HashMap;
use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_update;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::helpers::{nonempty, nonzero_u32};
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::UpdateItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::{TypeConfig, UpdateOptions};
use std::path::Path;
pub(super) fn build_update_options(
    title: String, body: String, status: String,
    priority: i32, raw_fields: HashMap<String, String>,
) -> UpdateOptions {
    let custom_fields = raw_fields.into_iter().map(|(k, v)| {
        let val = serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v));
        (k, val)
    }).collect();
    UpdateOptions {
        title: nonempty(title), body: nonempty(body),
        status: nonempty(status), priority: nonzero_u32(priority),
        custom_fields,
    }
}
pub(super) async fn do_update(
    project_path: &Path,
    item_type: &str,
    config: &TypeConfig,
    item_id: &str,
    hook_type: &str,
    hook_project_path: &str,
    hook_item_id: &str,
    hook_data: serde_json::Value,
    project_path_str: &str,
    options: UpdateOptions,
) -> UpdateItemResponse {
    match generic_update(project_path, item_type, config, item_id, options).await {
        Ok(item) => {
            maybe_run_post_hooks(
                project_path, hook_type, HookOperation::Update,
                hook_project_path, Some(hook_item_id), Some(hook_data), true,
            ).await;
            UpdateItemResponse {
                success: true, error: String::new(),
                item: Some(generic_item_to_proto(&item, item_type)),
            }
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path, hook_type, HookOperation::Update,
                hook_project_path, Some(hook_item_id), Some(hook_data), false,
            ).await;
            UpdateItemResponse {
                success: false, error: to_error_json(project_path_str, &e), item: None,
            }
        }
    }
}
