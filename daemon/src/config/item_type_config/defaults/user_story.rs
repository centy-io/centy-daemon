use super::super::types::{ItemTypeConfig, ItemTypeFeatures};
use crate::config::CentyConfig;
use mdstore::IdStrategy;

/// Build the default user stories config from the project's `CentyConfig`.
#[must_use]
pub fn default_user_story_config(config: &CentyConfig) -> ItemTypeConfig {
    ItemTypeConfig {
        name: "User Story".to_string(),
        icon: Some("person".to_string()),
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures {
            display_number: true,
            priority: true,
            soft_delete: true,
            assets: true,
            org_sync: true,
            move_item: true,
            duplicate: true,
        },
        statuses: Vec::new(),
        priority_levels: Some(config.priority_levels),
        custom_fields: Vec::new(),
        template: Some("template.md".to_string()),
        listed: true,
    }
}
