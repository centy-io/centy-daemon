use super::super::CentyConfig;
use super::types::{ItemTypeConfig, ItemTypeFeatures};
use mdstore::{CustomFieldDef, IdStrategy, TypeConfig, TypeFeatures};

/// Convert an `ItemTypeConfig` to mdstore's `TypeConfig` for storage operations.
///
/// The `icon`, `soft_delete`, and `template` fields are centy-daemon-only
/// metadata and are intentionally dropped in this conversion.
impl From<&ItemTypeConfig> for TypeConfig {
    fn from(config: &ItemTypeConfig) -> TypeConfig {
        TypeConfig {
            name: config.name.clone(),
            identifier: config.identifier,
            features: TypeFeatures {
                display_number: config.features.display_number,
                status: config.features.status,
                priority: config.features.priority,
                assets: config.features.assets,
                org_sync: config.features.org_sync,
                move_item: config.features.move_item,
                duplicate: config.features.duplicate,
            },
            statuses: config.statuses.clone(),
            default_status: config.default_status.clone(),
            priority_levels: config.priority_levels,
            custom_fields: config.custom_fields.clone(),
        }
    }
}

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
            status: false,
            priority: false,
            soft_delete: false,
            assets: true,
            org_sync: true,
            move_item: true,
            duplicate: false,
        },
        statuses: Vec::new(),
        default_status: None,
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

/// Default issue statuses used when no legacy `allowedStates` migration data is available.
pub const DEFAULT_ISSUE_STATUSES: &[&str] = &["open", "planning", "in-progress", "closed"];

/// Build the default issues config from the project's `CentyConfig`.
/// Statuses default to [`DEFAULT_ISSUE_STATUSES`]; callers that perform legacy
/// migration should overwrite `statuses` / `default_status` after calling this.
#[must_use]
pub fn default_issue_config(config: &CentyConfig) -> ItemTypeConfig {
    let statuses: Vec<String> = DEFAULT_ISSUE_STATUSES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let default_status = statuses.first().cloned();
    ItemTypeConfig {
        name: "Issue".to_string(),
        icon: Some("clipboard".to_string()),
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures {
            display_number: true,
            status: true,
            priority: true,
            soft_delete: true,
            assets: true,
            org_sync: true,
            move_item: true,
            duplicate: true,
        },
        statuses,
        default_status,
        priority_levels: Some(config.priority_levels),
        custom_fields: config.custom_fields.clone(),
        template: Some("template.md".to_string()),
    }
}
