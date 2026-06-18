use super::super::types::{ItemTypeConfig, ItemTypeFeatures};
use crate::config::CentyConfig;
use mdstore::IdStrategy;

/// Build the default epics config from the project's `CentyConfig`.
///
/// Epics are statusless — they have no `status` field and no status-related
/// logic applies to them.
#[must_use]
pub fn default_epic_config(config: &CentyConfig) -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Epic".to_string(),
        icon: Some("map".to_string()),
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
