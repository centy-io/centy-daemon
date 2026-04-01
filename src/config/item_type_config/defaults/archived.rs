use super::super::types::{ItemTypeConfig, ItemTypeFeatures};
use mdstore::{CustomFieldDef, IdStrategy};

/// Build the default archived config with hardcoded defaults.
///
/// The archived folder is a catch-all for items moved out of active view.
/// Items retain their content and metadata; `original_item_type` tracks the
/// source folder so they can be unarchived back to the correct location.
#[must_use]
pub fn default_archived_config() -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Archived".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures {
            display_number: false,
            priority: false,
            soft_delete: false,
            assets: true,
            org_sync: true,
            move_item: true,
            duplicate: false,
        },
        statuses: Vec::new(),
        priority_levels: None,
        custom_fields: vec![CustomFieldDef {
            name: "original_item_type".to_string(),
            field_type: "string".to_string(),
            required: false,
            default_value: None,
            enum_values: Vec::new(),
        }],
        template: None,
    }
}
