use std::path::Path;

use crate::config::item_type_config::{
    default_archived_config, default_doc_config, default_issue_config, read_item_type_config,
    ItemTypeRegistry,
};
use crate::config::read_config;
use crate::item::core::error::ItemError;
use mdstore::TypeConfig;

/// Resolve an item type string to its folder name and `TypeConfig`.
///
/// Resolution strategy:
/// 1. Build the `ItemTypeRegistry` and use `resolve()` (handles exact folder,
///    case-insensitive name, and case-insensitive plural matching).
/// 2. If registry lookup fails, try `read_item_type_config` for legacy projects.
/// 3. Fall back to built-in defaults for "issues"/"issue" and "docs"/"doc".
/// 4. Otherwise return `ItemError::ItemTypeNotFound`.
pub async fn resolve_item_type_config(
    project_path: &Path,
    item_type: &str,
) -> Result<(String, TypeConfig), ItemError> {
    // 1. Try registry lookup
    let registry = ItemTypeRegistry::build(project_path)
        .await
        .map_err(|e| ItemError::Custom(e.to_string()))?;

    if let Some((folder, config)) = registry.resolve(item_type) {
        return Ok((folder.clone(), config.clone()));
    }

    // 2. Try direct config read for legacy projects
    if let Ok(Some(config)) = read_item_type_config(project_path, item_type).await {
        return Ok((item_type.to_string(), config));
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
            Ok(("issues".to_string(), default_issue_config(&config)))
        }
        "docs" | "doc" => Ok(("docs".to_string(), default_doc_config())),
        "archived" | "archive" => Ok(("archived".to_string(), default_archived_config())),
        _ => Err(ItemError::ItemTypeNotFound(item_type.to_string())),
    }
}
