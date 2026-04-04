use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::item::generic::storage::generic_update;
use crate::registry::find_org_repo;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::helpers::{nonempty, nonzero_u32};
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::UpdateItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::{Filters, TypeConfig, UpdateOptions};
use std::collections::HashMap;
use std::path::Path;
pub(super) fn build_update_options(
    title: String,
    body: String,
    status: String,
    priority: i32,
    tags: Vec<String>,
    clear_tags: bool,
    raw_fields: HashMap<String, String>,
) -> UpdateOptions {
    let custom_fields = raw_fields
        .into_iter()
        .map(|(k, v)| {
            let val = serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v));
            (k, val)
        })
        .collect();
    let resolved_tags = if clear_tags {
        Some(vec![])
    } else if tags.is_empty() {
        None
    } else {
        Some(tags)
    };
    UpdateOptions {
        title: nonempty(title),
        body: nonempty(body),
        status: nonempty(status),
        priority: nonzero_u32(priority),
        tags: resolved_tags,
        custom_fields,
        comment: None,
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
    let result = match generic_update(project_path, item_type, config, item_id, options.clone()).await {
        Ok(item) => Ok(item),
        Err(ItemError::NotFound(_)) => {
            // Not found in project — try org repo fallback.
            update_in_org_repo(project_path_str, item_type, config, item_id, options).await
        }
        Err(e) => Err(e),
    };
    match result {
        Ok(item) => {
            maybe_run_post_hooks(
                project_path,
                hook_type,
                HookOperation::Update,
                hook_project_path,
                Some(hook_item_id),
                Some(hook_data),
                true,
            )
            .await;
            UpdateItemResponse {
                success: true,
                error: String::new(),
                item: Some(generic_item_to_proto(&item, item_type)),
            }
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                hook_type,
                HookOperation::Update,
                hook_project_path,
                Some(hook_item_id),
                Some(hook_data),
                false,
            )
            .await;
            UpdateItemResponse {
                success: false,
                error: to_error_json(project_path_str, &e),
                item: None,
            }
        }
    }
}

/// Attempt to update an item in the org repo.
///
/// Handles display-number resolution: if `item_id` parses as a positive integer
/// and the item type has `display_number` enabled, the org repo is scanned to
/// find the matching UUID before performing the update.
async fn update_in_org_repo(
    project_path_str: &str,
    item_type: &str,
    config: &TypeConfig,
    item_id: &str,
    options: UpdateOptions,
) -> Result<mdstore::Item, ItemError> {
    let Ok(Some(org_repo_path)) = find_org_repo(project_path_str).await else {
        return Err(ItemError::NotFound(item_id.to_string()));
    };
    let type_dir = Path::new(&org_repo_path).join(item_type);
    let resolved_id = resolve_id_in_type_dir(config, item_id, &type_dir).await?;
    Ok(mdstore::update(&type_dir, config, &resolved_id, options).await?)
}

/// Resolve a display-number string to a UUID within a given type directory.
///
/// If `item_id` parses as a positive integer and `display_number` is enabled,
/// the directory is scanned for an item with that display number.  Otherwise,
/// `item_id` is returned unchanged.
async fn resolve_id_in_type_dir(
    config: &TypeConfig,
    item_id: &str,
    type_dir: &Path,
) -> Result<String, ItemError> {
    if config.features.display_number {
        if let Ok(num) = item_id.parse::<u32>() {
            if num > 0 {
                let items = mdstore::list(type_dir, Filters::new().include_deleted()).await?;
                for item in items {
                    if item.frontmatter.display_number == Some(num) {
                        return Ok(item.id);
                    }
                }
                return Err(ItemError::NotFound(format!("display_number {num}")));
            }
        }
    }
    Ok(item_id.to_string())
}
