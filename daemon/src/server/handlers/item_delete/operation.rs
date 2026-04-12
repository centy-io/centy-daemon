use crate::hooks::HookOperation;
use crate::item::core::error::ItemError;
use crate::item::generic::storage::generic_delete;
use crate::registry::find_org_repo;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::DeleteItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::{Filters, TypeConfig};
use std::path::Path;
pub(super) async fn do_delete(
    project_path: &Path,
    item_type: &str,
    config: &TypeConfig,
    item_id: &str,
    force: bool,
    hook_type: &str,
    hook_project_path: &str,
    hook_item_id: &str,
    hook_data: serde_json::Value,
    project_path_str: &str,
) -> DeleteItemResponse {
    let result = match generic_delete(project_path, item_type, config, item_id, force).await {
        Ok(()) => Ok(()),
        Err(ItemError::NotFound(_)) => {
            // Not found in project — try org repo fallback.
            delete_in_org_repo(project_path_str, item_type, config, item_id, force).await
        }
        Err(e) => Err(e),
    };
    match result {
        Ok(()) => {
            maybe_run_post_hooks(
                project_path,
                hook_type,
                HookOperation::Delete,
                hook_project_path,
                Some(hook_item_id),
                Some(hook_data),
                true,
            )
            .await;
            DeleteItemResponse {
                success: true,
                error: String::new(),
            }
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                hook_type,
                HookOperation::Delete,
                hook_project_path,
                Some(hook_item_id),
                Some(hook_data),
                false,
            )
            .await;
            DeleteItemResponse {
                success: false,
                error: to_error_json(project_path_str, &e),
            }
        }
    }
}

/// Attempt to delete an item from the org repo.
///
/// Handles display-number resolution: if `item_id` parses as a positive integer
/// and the item type has `display_number` enabled, the org repo is scanned to
/// find the matching UUID before performing the deletion.
async fn delete_in_org_repo(
    project_path_str: &str,
    item_type: &str,
    config: &TypeConfig,
    item_id: &str,
    force: bool,
) -> Result<(), ItemError> {
    let Ok(Some(org_repo_path)) = find_org_repo(project_path_str).await else {
        return Err(ItemError::NotFound(item_id.to_string()));
    };
    let type_dir = Path::new(&org_repo_path).join(item_type);
    let resolved_id = resolve_id_in_type_dir(config, item_id, &type_dir).await?;
    Ok(mdstore::delete(&type_dir, &resolved_id, force).await?)
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
