use super::*;
use mdstore::IdStrategy;

#[test]
fn test_yaml_soft_delete_feature() {
    let yaml = "name: Bug\nidentifier: uuid\nfeatures:\n  displayNumber: false\n  status: true\n  priority: true\n  softDelete: true\n  assets: false\n  orgSync: false\n  move: false\n  duplicate: false\n";
    let config: ItemTypeConfig = serde_yaml::from_str(yaml).expect("Should deserialize");
    assert!(config.features.soft_delete);
}

#[test]
fn test_validate_item_type_config_valid() {
    let config = ItemTypeConfig {
        name: "Issue".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: vec!["open".to_string(), "closed".to_string()],
        default_status: Some("open".to_string()),
        priority_levels: Some(3),
        custom_fields: Vec::new(),
        template: None,
    };
    assert!(validate_item_type_config(&config).is_ok());
}

#[test]
fn test_validate_item_type_config_empty_name() {
    let config = ItemTypeConfig {
        name: String::new(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let result = validate_item_type_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name must not be empty"));
}

#[test]
fn test_validate_item_type_config_whitespace_name() {
    let config = ItemTypeConfig {
        name: "   ".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let result = validate_item_type_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_validate_item_type_config_zero_priority_levels() {
    let config = ItemTypeConfig {
        name: "Issue".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: Some(0),
        custom_fields: Vec::new(),
        template: None,
    };
    let result = validate_item_type_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("priorityLevels must be greater than 0"));
}
