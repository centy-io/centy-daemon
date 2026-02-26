mod loader;
pub use loader::load_user_config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum UserConfigError {
    #[error("Failed to read user config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse user config TOML: {0}")]
    Toml(#[from] toml::de::Error),
}
fn default_ignore_paths() -> Vec<String> {
    vec!["$TMPDIR".to_string(), "~/worktrees/*".to_string()]
}
/// Registry-scoped settings (`[registry]` table in the TOML file).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegistryConfig {
    #[serde(default = "default_ignore_paths")]
    pub ignore_paths: Vec<String>,
}
impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            ignore_paths: default_ignore_paths(),
        }
    }
}
/// Top-level user configuration, deserialized from `~/.centy/config.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UserConfig {
    #[serde(default)]
    pub registry: RegistryConfig,
}
/// Resolve the canonical path for the user config file.
#[must_use]
pub fn user_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".centy").join("config.toml"))
}
#[cfg(test)]
#[path = "../user_config_tests.rs"]
mod user_config_tests;
