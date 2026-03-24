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
#[path = "itc_tests_1.rs"]
mod itc_tests_1;
#[cfg(test)]
#[path = "itc_tests_10.rs"]
mod itc_tests_10;
#[cfg(test)]
#[path = "itc_tests_2.rs"]
mod itc_tests_2;
#[cfg(test)]
#[path = "itc_tests_3.rs"]
mod itc_tests_3;
#[cfg(test)]
#[path = "itc_tests_4.rs"]
mod itc_tests_4;
#[cfg(test)]
#[path = "itc_tests_5.rs"]
mod itc_tests_5;
#[cfg(test)]
#[path = "itc_tests_6.rs"]
mod itc_tests_6;
#[cfg(test)]
#[path = "itc_tests_7.rs"]
mod itc_tests_7;
#[cfg(test)]
#[path = "itc_tests_8.rs"]
mod itc_tests_8;
#[cfg(test)]
#[path = "itc_tests_9.rs"]
mod itc_tests_9;
