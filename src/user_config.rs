//! User-level global configuration loaded from `~/.centy/config.toml`.
//!
//! This file is optional; if it does not exist all fields fall back to their
//! `Default` values.  The schema is intentionally minimal — it establishes the
//! `[registry]` section skeleton so that future work (e.g. `ignore_paths` in
//! issue #204) has a stable home to land in.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum UserConfigError {
    #[error("Failed to read user config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse user config TOML: {0}")]
    Toml(#[from] toml::de::Error),
}

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

/// Registry-scoped settings (`[registry]` table in the TOML file).
///
/// Currently empty; `ignore_paths` (issue #204) and other fields will be
/// added here.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegistryConfig {}

/// Top-level user configuration, deserialized from
/// `~/.centy/config.toml`.
///
/// All fields are optional at the TOML level; missing fields resolve to their
/// `Default` values.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UserConfig {
    /// Registry-level settings (`[registry]` section).
    #[serde(default)]
    pub registry: RegistryConfig,
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Resolve the canonical path for the user config file (`~/.centy/config.toml`).
///
/// Co-located with the rest of the user-scoped centy data (`projects.json`,
/// `logs/`) so everything user-level lives under one directory.
#[must_use]
pub fn user_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".centy").join("config.toml"))
}

/// Load the user configuration from `~/.centy/config.toml`.
///
/// Returns `Ok(UserConfig::default())` if the file does not exist so callers
/// never need to handle the "absent file" case specially.
///
/// # Errors
///
/// Returns [`UserConfigError`] if the file exists but cannot be read or parsed.
pub fn load_user_config() -> Result<UserConfig, UserConfigError> {
    let path = match user_config_path() {
        Some(p) => p,
        None => {
            warn!("Could not determine user config directory; using defaults");
            return Ok(UserConfig::default());
        }
    };

    if !path.exists() {
        debug!(
            "User config not found at {}; using defaults",
            path.display()
        );
        return Ok(UserConfig::default());
    }

    let content = std::fs::read_to_string(&path)?;
    let config: UserConfig = toml::from_str(&content)?;
    debug!("Loaded user config from {}", path.display());
    Ok(config)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// When the config file is absent `load_user_config` should return the
    /// default `UserConfig` without error.
    #[test]
    fn test_defaults_when_file_absent() {
        // Point HOME / config dir somewhere empty via the function under test
        // by verifying that a missing path returns defaults.
        //
        // We cannot easily override dirs::home_dir() in a unit test, so
        // instead we test the parsing path directly with an empty TOML string.
        let config: UserConfig = toml::from_str("").expect("empty TOML should parse");
        assert_eq!(config, UserConfig::default());
    }

    #[test]
    fn test_user_config_default() {
        let cfg = UserConfig::default();
        assert_eq!(cfg.registry, RegistryConfig::default());
    }

    #[test]
    fn test_empty_toml_produces_defaults() {
        let cfg: UserConfig = toml::from_str("").expect("Should parse empty TOML");
        assert_eq!(cfg, UserConfig::default());
    }

    #[test]
    fn test_registry_section_only() {
        let toml_str = "[registry]\n";
        let cfg: UserConfig = toml::from_str(toml_str).expect("Should parse [registry] section");
        assert_eq!(cfg.registry, RegistryConfig::default());
    }

    #[test]
    fn test_roundtrip_serialization() {
        let cfg = UserConfig::default();
        let serialized = toml::to_string(&cfg).expect("Should serialize");
        let deserialized: UserConfig = toml::from_str(&serialized).expect("Should deserialize");
        assert_eq!(cfg, deserialized);
    }

    #[test]
    fn test_load_user_config_absent_file() {
        // Construct a path that definitely doesn't exist and verify the
        // file-absent branch via load_user_config by calling the underlying
        // logic (existence check + parse).
        let dir = tempdir().expect("tempdir");
        let non_existent = dir.path().join("config.toml");
        assert!(!non_existent.exists());

        // Simulate the "file absent" branch manually (since we can't override
        // dirs::home_dir() at runtime without mocking infrastructure).
        let content_result: Result<String, std::io::Error> = fs::read_to_string(&non_existent);
        assert!(content_result.is_err()); // file not found
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempdir().expect("tempdir");
        let config_path = dir.path().join("config.toml");

        let toml_content = "# centy user config\n\n[registry]\n";
        fs::write(&config_path, toml_content).expect("write config");

        let content = fs::read_to_string(&config_path).expect("read config");
        let cfg: UserConfig = toml::from_str(&content).expect("parse config");
        assert_eq!(cfg, UserConfig::default());
    }
}
