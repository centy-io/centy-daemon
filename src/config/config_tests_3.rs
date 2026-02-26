use super::*;
use tokio::fs;

#[tokio::test]
async fn test_read_config_normalizes_missing_hooks() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let centy_dir = temp_dir.path().join(".centy");
    fs::create_dir_all(&centy_dir)
        .await
        .expect("Should create .centy dir");
    let config_without_hooks = r#"{"priorityLevels": 3, "customFields": [], "defaults": {}, "allowedStates": ["open", "planning", "in-progress", "closed"], "defaultState": "open", "stateColors": {}, "priorityColors": {}}"#;
    let config_path = centy_dir.join("config.json");
    fs::write(&config_path, config_without_hooks)
        .await
        .expect("Should write config");
    let config = read_config(temp_dir.path())
        .await
        .expect("Should read")
        .expect("Config should exist");
    assert!(config.hooks.is_empty());
    let raw = fs::read_to_string(&config_path)
        .await
        .expect("Should read file");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("Should parse");
    assert!(
        value.as_object().unwrap().contains_key("hooks"),
        "config.json should now contain the hooks key"
    );
    assert!(
        !value.as_object().unwrap().contains_key("llm"),
        "config.json should not contain llm config"
    );
}

#[tokio::test]
async fn test_read_config_does_not_rewrite_when_hooks_present_and_no_deprecated_fields() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let centy_dir = temp_dir.path().join(".centy");
    fs::create_dir_all(&centy_dir)
        .await
        .expect("Should create .centy dir");
    let config_with_hooks = r#"{"customFields": [], "defaults": {}, "hooks": [], "priorityColors": {}, "priorityLevels": 3, "stateColors": {}}"#;
    let config_path = centy_dir.join("config.json");
    fs::write(&config_path, config_with_hooks)
        .await
        .expect("Should write config");
    let config = read_config(temp_dir.path())
        .await
        .expect("Should read")
        .expect("Config should exist");
    assert!(config.hooks.is_empty());
    let raw = fs::read_to_string(&config_path)
        .await
        .expect("Should read file");
    assert_eq!(raw, config_with_hooks);
}

#[tokio::test]
async fn test_read_config_strips_legacy_allowed_states() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let centy_dir = temp_dir.path().join(".centy");
    fs::create_dir_all(&centy_dir)
        .await
        .expect("Should create .centy dir");
    let config_with_allowed_states = r#"{"allowedStates": ["open", "planning", "in-progress", "closed"], "customFields": [], "defaults": {}, "hooks": [], "priorityColors": {}, "priorityLevels": 3, "stateColors": {}}"#;
    let config_path = centy_dir.join("config.json");
    fs::write(&config_path, config_with_allowed_states)
        .await
        .expect("Should write config");
    let config = read_config(temp_dir.path())
        .await
        .expect("Should read")
        .expect("Config should exist");
    assert_eq!(config.allowed_states.len(), 4);
    let raw = fs::read_to_string(&config_path)
        .await
        .expect("Should read file");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("Should parse");
    assert!(
        !value.as_object().unwrap().contains_key("allowedStates"),
        "config.json should no longer contain allowedStates after rewrite"
    );
}

#[tokio::test]
async fn test_read_config_flat_format_works() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let centy_dir = temp_dir.path().join(".centy");
    fs::create_dir_all(&centy_dir)
        .await
        .expect("Should create .centy dir");
    let flat_config = r#"{"version": "0.0.1", "priorityLevels": 5, "customFields": [], "defaults": {}, "allowedStates": ["open", "closed"], "defaultState": "open", "stateColors": {}, "priorityColors": {}, "hooks": []}"#;
    let config_path = centy_dir.join("config.json");
    fs::write(&config_path, flat_config)
        .await
        .expect("Should write config");
    let config = read_config(temp_dir.path())
        .await
        .expect("Should read")
        .expect("Config should exist");
    assert_eq!(config.version, Some("0.0.1".to_string()));
    assert_eq!(config.priority_levels, 5);
}

#[test]
fn test_hooks_always_serialized_even_when_empty() {
    let config = CentyConfig::default();
    let json = serde_json::to_string(&config).expect("Should serialize");
    assert!(
        json.contains("\"hooks\""),
        "hooks key should be present in serialized JSON even when empty"
    );
}

#[test]
fn test_workspace_config_default() {
    let ws = WorkspaceConfig::default();
    assert!(ws.update_status_on_open.is_none());
}

#[test]
fn test_workspace_config_serialization_skips_none() {
    let ws = WorkspaceConfig::default();
    let json = serde_json::to_string(&ws).expect("Should serialize");
    assert_eq!(json, "{}");
}
