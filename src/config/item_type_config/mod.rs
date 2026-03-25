mod convert;
mod defaults;
mod io;
mod migrate;
mod registry;
mod types;

pub use convert::{default_archived_config, default_issue_config};
pub use defaults::{default_comment_config, default_doc_config, validate_item_type_config};
pub use io::{
    discover_item_types, discover_item_types_map, read_item_type_config,
    read_legacy_allowed_states, write_item_type_config,
};
pub use migrate::{migrate_strip_status_feature, migrate_to_item_type_configs};
pub use registry::ItemTypeRegistry;
pub use types::{ItemTypeConfig, ItemTypeFeatures};

#[cfg(test)]
#[path = "default_issue_config.rs"]
mod default_issue_config;
#[cfg(test)]
#[path = "item_type_config_field_validation.rs"]
mod item_type_config_field_validation;
#[cfg(test)]
#[path = "item_type_config_migration.rs"]
mod item_type_config_migration;
#[cfg(test)]
#[path = "item_type_config_validation.rs"]
mod item_type_config_validation;
#[cfg(test)]
#[path = "item_type_config_yaml_serialization.rs"]
mod item_type_config_yaml_serialization;
#[cfg(test)]
#[path = "item_type_registry_build.rs"]
mod item_type_registry_build;
#[cfg(test)]
#[path = "item_type_registry_error_handling.rs"]
mod item_type_registry_error_handling;
#[cfg(test)]
#[path = "item_type_registry_lookup.rs"]
mod item_type_registry_lookup;
#[cfg(test)]
#[path = "item_type_registry_resolve.rs"]
mod item_type_registry_resolve;
#[cfg(test)]
#[path = "type_config_conversion.rs"]
mod type_config_conversion;
