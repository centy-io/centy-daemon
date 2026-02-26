use std::collections::HashMap;
use std::path::Path;
use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_create;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::CreateItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::{CreateOptions, TypeConfig};
pub(super) async fn do_create(
    project_path: &Path,
    item_type: &str,
    config: &TypeConfig,
    hook_type: &str,
    hook_project_path: &str,
    hook_data: serde_json::Value,
    project_path_str: &str,
    options: CreateOptions,
) -> CreateItemResponse {
    match generic_create(project_path, item_type, config, options).await {
        Ok(item) => {
            maybe_run_post_hooks(
                project_path, hook_type, HookOperation::Create,
                hook_project_path, Some(&item.id), Some(hook_data), true,
            ).await;
            CreateItemResponse {
                success: true, error: String::new(),
                item: Some(generic_item_to_proto(&item, item_type)),
            }
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path, hook_type, HookOperation::Create,
                hook_project_path, None, Some(hook_data), false,
            ).await;
            CreateItemResponse { success: false, error: to_error_json(project_path_str, &e), item: None }
        }
    }
}
pub(super) fn build_options(
    title: String, body: String, status: Option<String>,
    priority: Option<u32>, custom_fields: HashMap<String, String>,
) -> CreateOptions {
    let custom_fields = custom_fields.into_iter().map(|(k, v)| {
        let val = serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v));
        (k, val)
    }).collect();
    CreateOptions { title, body, id: None, status, priority, custom_fields }
}
