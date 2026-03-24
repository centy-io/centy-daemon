use super::types::{ItemTypeConfig, ItemTypeFeatures};
use mdstore::{CustomFieldDef, IdStrategy};

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

/// Build the default comments config with hardcoded defaults.
///
/// Comments are a lightweight built-in item type for annotating other items.
/// They use UUID identifiers and store `item_id`, `item_type`, and `author`
/// as custom fields. All optional features are disabled.
#[must_use]
pub fn default_comment_config() -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Comment".to_string(),
        icon: Some("chat-bubble".to_string()),
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures {
            display_number: false,
            priority: false,
            soft_delete: false,
            assets: false,
            org_sync: false,
            move_item: false,
            duplicate: false,
        },
        statuses: Vec::new(),
        priority_levels: None,
        custom_fields: vec![
            CustomFieldDef {
                name: "item_id".to_string(),
                field_type: "string".to_string(),
                required: false,
                default_value: None,
                enum_values: Vec::new(),
            },
            CustomFieldDef {
                name: "item_type".to_string(),
                field_type: "string".to_string(),
                required: false,
                default_value: None,
                enum_values: Vec::new(),
            },
            CustomFieldDef {
                name: "author".to_string(),
                field_type: "string".to_string(),
                required: false,
                default_value: None,
                enum_values: Vec::new(),
            },
        ],
        template: None,
    }
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
            soft_delete: false,
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
