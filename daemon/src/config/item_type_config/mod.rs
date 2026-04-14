mod convert;
mod defaults;
mod io;
mod migrate;
mod registry;
mod types;

pub use defaults::{
    default_archived_config, default_comment_config, default_doc_config, default_epic_config,
    default_issue_config,
};
pub use io::{
    discover_item_types, discover_item_types_map, read_item_type_config,
    read_legacy_allowed_states, write_item_type_config,
};
pub use migrate::{migrate_strip_status_feature, migrate_to_item_type_configs};
pub use registry::ItemTypeRegistry;
pub use types::{ItemTypeConfig, ItemTypeFeatures};

#[cfg(test)]
#[path = "default_configs_tests.rs"]
mod default_configs_tests;
#[cfg(test)]
#[path = "item_type_config_migration_tests.rs"]
mod item_type_config_migration_tests;
#[cfg(test)]
#[path = "item_type_config_validation_tests.rs"]
mod item_type_config_validation_tests;
#[cfg(test)]
#[path = "item_type_config_yaml_serialization_tests.rs"]
mod item_type_config_yaml_serialization_tests;
#[cfg(test)]
#[path = "item_type_registry_build_tests.rs"]
mod item_type_registry_build_tests;
#[cfg(test)]
#[path = "item_type_registry_error_handling_tests.rs"]
mod item_type_registry_error_handling_tests;
#[cfg(test)]
#[path = "item_type_registry_lookup_tests.rs"]
mod item_type_registry_lookup_tests;
#[cfg(test)]
#[path = "item_type_registry_resolve_tests.rs"]
mod item_type_registry_resolve_tests;
#[cfg(test)]
#[path = "type_config_conversion_tests.rs"]
mod type_config_conversion_tests;
