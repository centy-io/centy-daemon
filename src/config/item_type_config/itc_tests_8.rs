#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default
)]
use super::*;
use mdstore::IdStrategy;

#[test]
fn test_validate_item_type_config_none_priority_levels_ok() {
    let config = ItemTypeConfig {
        name: "Doc".to_string(),
        icon: None,
        identifier: IdStrategy::Slug,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    assert!(validate_item_type_config(&config).is_ok());
}

#[test]
fn test_validate_item_type_config_empty_status_name() {
    let config = ItemTypeConfig {
        name: "Issue".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: vec!["open".to_string(), String::new()],
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let result = validate_item_type_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("status names must not be empty"));
}

#[test]
fn test_validate_item_type_config_whitespace_status_name() {
    let config = ItemTypeConfig {
        name: "Issue".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: vec!["open".to_string(), "  ".to_string()],
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let result = validate_item_type_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_validate_item_type_config_no_statuses_no_default_ok() {
    let config = ItemTypeConfig {
        name: "Doc".to_string(),
        icon: None,
        identifier: IdStrategy::Slug,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    assert!(validate_item_type_config(&config).is_ok());
}
