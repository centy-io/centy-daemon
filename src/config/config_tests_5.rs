use super::*;
use std::collections::HashMap;

#[test]
fn test_extra_roundtrip() {
    let mut extra = HashMap::new();
    extra.insert("team".to_string(), serde_json::json!("backend"));
    extra.insert("sprint".to_string(), serde_json::json!(42i64));
    let config = CentyConfig {
        extra,
        ..CentyConfig::default()
    };
    let json = serde_json::to_string(&config).expect("serialize");
    let de: CentyConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(de.extra.get("team").and_then(serde_json::Value::as_str), Some("backend"));
    assert_eq!(de.extra.get("sprint").and_then(serde_json::Value::as_i64), Some(42i64));
}

#[test]
fn test_system_keys_not_captured_in_extra() {
    let json = r#"{"priorityLevels":5,"version":"1.0.0","team":"backend"}"#;
    let config: CentyConfig = serde_json::from_str(json).expect("deserialize");
    assert_eq!(config.priority_levels, 5);
    assert_eq!(config.version, Some("1.0.0".to_string()));
    assert_eq!(config.extra.get("team").and_then(serde_json::Value::as_str), Some("backend"));
    assert!(!config.extra.contains_key("priorityLevels"));
    assert!(!config.extra.contains_key("version"));
}

#[test]
fn test_is_system_key_exact() {
    assert!(is_system_key("version"));
    assert!(is_system_key("priorityLevels"));
    assert!(is_system_key("workspace"));
    assert!(is_system_key("cleanup"));
    assert!(!is_system_key("team"));
    assert!(!is_system_key("myCustomKey"));
}

#[test]
fn test_is_system_key_prefix() {
    assert!(is_system_key("workspace.updateStatusOnOpen"));
    assert!(is_system_key("workspace.anything"));
    assert!(!is_system_key("workspaceFoo"));
}

#[test]
fn test_default_extra_is_empty() {
    let config = CentyConfig::default();
    assert!(config.extra.is_empty());
}

#[test]
fn test_extra_preserved_through_json_with_nested_value() {
    let mut extra = HashMap::new();
    extra.insert("metadata".to_string(), serde_json::json!({"env": "prod", "count": 3i64}));
    let config = CentyConfig {
        extra,
        ..CentyConfig::default()
    };
    let json = serde_json::to_string(&config).expect("serialize");
    let de: CentyConfig = serde_json::from_str(&json).expect("deserialize");
    let meta = de.extra.get("metadata").expect("metadata key");
    assert_eq!(meta["env"], "prod");
    assert_eq!(meta["count"], 3i64);
}
