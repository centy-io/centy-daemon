use super::super::types::{ItemTypeConfig, ItemTypeFeatures};
use mdstore::{CustomFieldDef, IdStrategy};

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
