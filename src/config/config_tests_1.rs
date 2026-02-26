use super::*;

#[test]
fn test_default_priority_levels() {
    assert_eq!(default_priority_levels(), 3);
}

#[test]
fn test_default_allowed_states() {
    let states = default_allowed_states();
    assert_eq!(states.len(), 4);
    assert!(states.contains(&"open".to_string()));
    assert!(states.contains(&"planning".to_string()));
    assert!(states.contains(&"in-progress".to_string()));
    assert!(states.contains(&"closed".to_string()));
}

#[test]
fn test_centy_config_default() {
    let config = CentyConfig::default();
    assert!(config.version.is_none());
    assert_eq!(config.priority_levels, 3);
    assert!(config.custom_fields.is_empty());
    assert!(config.defaults.is_empty());
    assert_eq!(config.allowed_states.len(), 4);
    assert!(config.state_colors.is_empty());
    assert!(config.priority_colors.is_empty());
    assert!(config.custom_link_types.is_empty());
    assert!(config.hooks.is_empty());
    assert!(config.workspace.update_status_on_open.is_none());
}

#[test]
fn test_centy_config_effective_version_with_version() {
    let mut config = CentyConfig::default();
    config.version = Some("1.0.0".to_string());
    assert_eq!(config.effective_version(), "1.0.0");
}

#[test]
fn test_centy_config_effective_version_without_version() {
    let config = CentyConfig::default();
    assert_eq!(config.effective_version(), crate::utils::CENTY_VERSION);
}

#[test]
fn test_centy_config_serialization_deserialization() {
    let mut config = CentyConfig::default();
    config.version = Some("1.2.3".to_string());
    config.priority_levels = 5;
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: CentyConfig = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.version, Some("1.2.3".to_string()));
    assert_eq!(deserialized.priority_levels, 5);
}

#[test]
fn test_centy_config_json_uses_camel_case() {
    let config = CentyConfig::default();
    let json = serde_json::to_string(&config).expect("Should serialize");
    assert!(json.contains("priorityLevels"));
    assert!(json.contains("customFields"));
    assert!(json.contains("stateColors"));
    assert!(json.contains("priorityColors"));
    assert!(!json.contains("allowedStates"));
    assert!(!json.contains("priority_levels"));
    assert!(!json.contains("custom_fields"));
    assert!(!json.contains("allowed_states"));
}

#[test]
fn test_custom_field_def_serialization() {
    let field = mdstore::CustomFieldDef {
        name: "environment".to_string(),
        field_type: "enum".to_string(),
        required: true,
        default_value: Some("dev".to_string()),
        enum_values: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
    };
    let json = serde_json::to_string(&field).expect("Should serialize");
    let deserialized: mdstore::CustomFieldDef =
        serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.name, "environment");
    assert_eq!(deserialized.field_type, "enum");
    assert!(deserialized.required);
    assert_eq!(deserialized.default_value, Some("dev".to_string()));
    assert_eq!(deserialized.enum_values.len(), 3);
}
