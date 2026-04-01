use super::super::types::{ItemTypeConfig, ItemTypeFeatures};
use crate::config::CentyConfig;
use mdstore::IdStrategy;

/// Default issue statuses used when no legacy `allowedStates` migration data is available.
pub const DEFAULT_ISSUE_STATUSES: &[&str] = &["open", "planning", "in-progress", "closed"];

/// Build the default issues config from the project's `CentyConfig`.
/// Statuses default to [`DEFAULT_ISSUE_STATUSES`]; callers that perform legacy
/// migration should overwrite `statuses` after calling this.
#[must_use]
pub fn default_issue_config(config: &CentyConfig) -> ItemTypeConfig {
    let statuses: Vec<String> = DEFAULT_ISSUE_STATUSES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    ItemTypeConfig {
        name: "Issue".to_string(),
        icon: Some("clipboard".to_string()),
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
        custom_fields: config.custom_fields.clone(),
        template: Some("template.md".to_string()),
    }
}
