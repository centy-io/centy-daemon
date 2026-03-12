use super::types::{ItemTypeConfig, ItemTypeFeatures};
use mdstore::IdStrategy;

/// Validate an `ItemTypeConfig` for correctness.
///
/// Checks:
/// - `name` must not be empty or whitespace-only.
/// - `priorityLevels` must be > 0 when present.
/// - Every value in `statuses` must be non-empty (after trimming).
pub fn validate_item_type_config(config: &ItemTypeConfig) -> Result<(), String> {
    if config.name.trim().is_empty() {
        return Err("name must not be empty".to_string());
    }
    if let Some(levels) = config.priority_levels {
        if levels == 0 {
            return Err("priorityLevels must be greater than 0".to_string());
        }
    }
    for status in &config.statuses {
        if status.trim().is_empty() {
            return Err("status names must not be empty".to_string());
        }
    }
    Ok(())
}

/// Build the default docs config with hardcoded defaults.
#[must_use]
pub fn default_doc_config() -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Doc".to_string(),
        icon: Some("document".to_string()),
        identifier: IdStrategy::Slug,
        features: ItemTypeFeatures {
            display_number: false,
            priority: false,
            assets: false,
            org_sync: true,
            move_item: true,
            duplicate: true,
        },
        statuses: Vec::new(),
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    }
}
