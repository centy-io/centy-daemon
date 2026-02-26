mod io;
#[allow(dead_code)]
pub mod item_type_config;
pub mod migrate;
mod types;
#[allow(unused_imports)]
pub use io::{
    get_project_title, read_config, read_project_metadata, set_project_title, write_config,
    write_project_metadata,
};
#[allow(unused_imports)]
pub use types::{
    default_allowed_states, default_priority_levels, CentyConfig, ProjectMetadata, WorkspaceConfig,
};
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "config_tests_1.rs"]
mod config_tests_1;
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "config_tests_2.rs"]
mod config_tests_2;
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "config_tests_3.rs"]
mod config_tests_3;
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "config_tests_4.rs"]
mod config_tests_4;
