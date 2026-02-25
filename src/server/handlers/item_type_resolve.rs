use std::path::Path;

use crate::config::item_type_config::{
    default_archived_config, default_doc_config, default_issue_config, read_item_type_config,
    ItemTypeRegistry,
};
use crate::config::read_config;
use crate::item::core::error::ItemError;
use crate::item::generic::storage::generic_get_by_display_number;
use mdstore::TypeConfig;

/// Resolve an item type string to its folder name and `TypeConfig`.
///
/// Resolution strategy:
/// 1. Build the `ItemTypeRegistry` and use `resolve()` (handles exact folder,
///    case-insensitive name, and case-insensitive plural matching).
/// 2. If registry lookup fails, try `read_item_type_config` for legacy projects.
/// 3. Fall back to built-in defaults for "issues"/"issue" and "docs"/"doc".
/// 4. Otherwise return `ItemError::ItemTypeNotFound`.
///
/// Returns `TypeConfig` (mdstore's type) for use with storage operations.
/// The `ItemTypeConfig` → `TypeConfig` conversion drops centy-only fields
/// (`icon`, `soft_delete`, `template`) which are not needed for storage.
pub async fn resolve_item_type_config(
    project_path: &Path,
    item_type: &str,
) -> Result<(String, TypeConfig), ItemError> {
    // 1. Try registry lookup
    let registry = ItemTypeRegistry::build(project_path)
        .await
        .map_err(|e| ItemError::Custom(e.to_string()))?;

    if let Some((folder, item_config)) = registry.resolve(item_type) {
        return Ok((folder.clone(), TypeConfig::from(item_config)));
    }

    // 2. Try direct config read for legacy projects
    if let Ok(Some(item_config)) = read_item_type_config(project_path, item_type).await {
        return Ok((item_type.to_string(), TypeConfig::from(&item_config)));
    }

    // 3. Built-in fallbacks
    let lower = item_type.to_lowercase();
    match lower.as_str() {
        "issues" | "issue" => {
            let config = read_config(project_path)
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
            Ok((
                "issues".to_string(),
                TypeConfig::from(&default_issue_config(&config)),
            ))
        }
        "docs" | "doc" => Ok((
            "docs".to_string(),
            TypeConfig::from(&default_doc_config()),
        )),
        "archived" | "archive" => Ok((
            "archived".to_string(),
            TypeConfig::from(&default_archived_config()),
        )),
        _ => Err(ItemError::ItemTypeNotFound(item_type.to_string())),
    }
}

/// Resolve a display-number string to the actual item UUID.
///
/// If `id` is a pure positive-integer string (e.g. "1") and the item type has
/// the `display_number` feature enabled, this looks up the item by display
/// number and returns its real UUID.  Otherwise returns `id` unchanged.
pub async fn resolve_item_id(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    id: &str,
) -> Result<String, ItemError> {
    if config.features.display_number {
        if let Ok(num) = id.parse::<u32>() {
            if num > 0 {
                let item = generic_get_by_display_number(project_path, folder, config, num).await?;
                return Ok(item.id);
            }
        }
    }
    Ok(id.to_string())
}
