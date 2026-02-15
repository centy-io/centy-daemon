use std::path::Path;

use crate::config::item_type_config::{
    default_doc_config, default_issue_config, read_item_type_config, ItemTypeConfig,
};
use crate::config::read_config;
use crate::hooks::HookItemType;
use crate::item::core::error::ItemError;

/// Normalize common item type aliases to canonical plural form.
pub fn normalize_item_type(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "issue" | "issues" => "issues".to_string(),
        "doc" | "docs" => "docs".to_string(),
        other => other.to_string(),
    }
}

/// Resolve an item type string to its `ItemTypeConfig`.
///
/// For built-in types ("issues", "docs"), returns hardcoded configs.
/// For custom types, reads from `.centy/<folder>/config.yaml`.
pub async fn resolve_item_type_config(
    project_path: &Path,
    item_type: &str,
) -> Result<ItemTypeConfig, ItemError> {
    let normalized = normalize_item_type(item_type);
    match normalized.as_str() {
        "issues" => {
            let config = read_config(project_path)
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
            Ok(default_issue_config(&config))
        }
        "docs" => Ok(default_doc_config()),
        other => match read_item_type_config(project_path, other).await {
            Ok(Some(config)) => Ok(config),
            Ok(None) => Err(ItemError::ItemTypeNotFound(other.to_string())),
            Err(e) => Err(ItemError::Custom(e.to_string())),
        },
    }
}

/// Map an item type string to a `HookItemType` for hook dispatch.
pub fn resolve_hook_item_type(item_type: &str) -> HookItemType {
    let normalized = normalize_item_type(item_type);
    match normalized.as_str() {
        "issues" => HookItemType::Issue,
        "docs" => HookItemType::Doc,
        _ => HookItemType::Issue, // fallback
    }
}
