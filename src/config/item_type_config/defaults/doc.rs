use super::super::types::{ItemTypeConfig, ItemTypeFeatures};
use mdstore::IdStrategy;

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
