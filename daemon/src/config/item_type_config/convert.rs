use super::types::ItemTypeConfig;
use mdstore::{TypeConfig, TypeFeatures};

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
                status: !config.statuses.is_empty(),
                priority: config.features.priority,
                assets: config.features.assets,
                org_sync: config.features.org_sync,
                move_item: config.features.move_item,
                duplicate: config.features.duplicate,
            },
            statuses: config.statuses.clone(),
            default_status: config.statuses.first().cloned(),
            priority_levels: config.priority_levels,
            custom_fields: config.custom_fields.clone(),
        }
    }
}
