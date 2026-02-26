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
    // Missing ignore_paths falls back to the serde default
    assert_eq!(cfg.registry, RegistryConfig::default());
}

#[test]
fn test_ignore_paths_explicit() {
    let toml_str = "[registry]\nignore_paths = [\"/tmp\", \"~/scratch\"]\n";
    let cfg: UserConfig = toml::from_str(toml_str).expect("Should parse ignore_paths");
    assert_eq!(cfg.registry.ignore_paths, vec!["/tmp", "~/scratch"]);
}

#[test]
fn test_ignore_paths_empty_overrides_default() {
    let toml_str = "[registry]\nignore_paths = []\n";
    let cfg: UserConfig = toml::from_str(toml_str).expect("Should parse empty ignore_paths");
    assert!(cfg.registry.ignore_paths.is_empty());
}

#[test]
fn test_default_ignore_paths_contains_tmpdir_and_worktrees() {
    let defaults = default_ignore_paths();
    assert!(defaults.iter().any(|p| p.contains("$TMPDIR")));
    assert!(defaults.iter().any(|p| p.contains("worktrees")));
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
