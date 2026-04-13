use super::super::types::{ItemTypeConfig, ItemTypeFeatures};
use crate::config::CentyConfig;
use mdstore::IdStrategy;

/// Default user story statuses for agile/product workflows.
pub const DEFAULT_USER_STORY_STATUSES: &[&str] = &["backlog", "ready", "in-progress", "done"];

/// Build the default user stories config from the project's `CentyConfig`.
#[must_use]
pub fn default_user_story_config(config: &CentyConfig) -> ItemTypeConfig {
    let statuses: Vec<String> = DEFAULT_USER_STORY_STATUSES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
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
        statuses,
        priority_levels: Some(config.priority_levels),
        custom_fields: Vec::new(),
        template: Some("template.md".to_string()),
        listed: true,
    }
}
