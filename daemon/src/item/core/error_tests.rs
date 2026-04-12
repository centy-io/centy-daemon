#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

#[test]
fn test_item_error_from_store_error_not_found() {
    let store_err = mdstore::StoreError::NotFound("item-abc".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::NotFound(_)));
    assert_eq!(format!("{err}"), "Item not found: item-abc");
}

#[test]
fn test_item_error_from_store_error_validation() {
    let store_err = mdstore::StoreError::ValidationError("bad value".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::ValidationError(_)));
}

#[test]
fn test_item_error_from_store_error_yaml() {
    let store_err = mdstore::StoreError::YamlError("bad yaml".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::YamlError(_)));
}

#[test]
fn test_item_error_from_store_error_frontmatter() {
    let store_err = mdstore::StoreError::FrontmatterError("bad frontmatter".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::FrontmatterError(_)));
}

#[test]
fn test_item_error_from_store_error_item_type_not_found() {
    let store_err = mdstore::StoreError::ItemTypeNotFound("issues".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::ItemTypeNotFound(_)));
}

#[test]
fn test_item_error_from_store_error_feature_not_enabled() {
    let store_err = mdstore::StoreError::FeatureNotEnabled("comments".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::FeatureNotEnabled(_)));
}

#[test]
fn test_item_error_from_store_error_already_deleted() {
    let store_err = mdstore::StoreError::AlreadyDeleted("item-xyz".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::AlreadyDeleted(_)));
}

#[test]
fn test_item_error_from_store_error_not_deleted() {
    let store_err = mdstore::StoreError::NotDeleted("item-xyz".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::NotDeleted(_)));
}

#[test]
fn test_item_error_from_store_error_invalid_status() {
    let store_err = mdstore::StoreError::InvalidStatus {
        status: "bad".to_string(),
        allowed: vec!["open".to_string()],
    };
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::InvalidStatus { .. }));
}

#[test]
fn test_item_error_from_store_error_invalid_priority() {
    let store_err = mdstore::StoreError::InvalidPriority {
        priority: 5,
        max: 3,
    };
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::InvalidPriority { .. }));
}

#[test]
fn test_item_error_from_store_error_already_exists() {
    let store_err = mdstore::StoreError::AlreadyExists("item-abc".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::AlreadyExists(_)));
}

#[test]
fn test_item_error_from_store_error_is_deleted() {
    let store_err = mdstore::StoreError::IsDeleted("item-abc".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::IsDeleted(_)));
}

#[test]
fn test_item_error_from_store_error_same_location() {
    let store_err = mdstore::StoreError::SameLocation;
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::SameProject));
}

#[test]
fn test_item_error_from_store_error_custom() {
    let store_err = mdstore::StoreError::Custom("custom error".to_string());
    let err = ItemError::from(store_err);
    assert!(matches!(err, ItemError::Custom(_)));
}

#[test]
fn test_item_error_from_config_error_yaml() {
    // Create a YAML error by parsing invalid YAML
    let yaml_err = serde_yaml::from_str::<serde_json::Value>(":\ninvalid").unwrap_err();
    let config_err = mdstore::ConfigError::YamlError(yaml_err);
    let err = ItemError::from(config_err);
    assert!(matches!(err, ItemError::YamlError(_)));
}

#[test]
fn test_item_error_same_project_display() {
    let err = ItemError::SameProject;
    assert_eq!(format!("{err}"), "Cannot move item to same project");
}

#[test]
fn test_item_error_target_not_initialized_display() {
    let err = ItemError::TargetNotInitialized;
    assert_eq!(format!("{err}"), "Target project not initialized");
}

#[test]
fn test_item_error_custom() {
    let err = ItemError::custom("something went wrong");
    assert!(matches!(err, ItemError::Custom(_)));
    assert_eq!(format!("{err}"), "something went wrong");
}

#[test]
fn test_item_error_not_found() {
    let err = ItemError::not_found("issue-123");
    assert!(matches!(err, ItemError::NotFound(_)));
    assert_eq!(format!("{err}"), "Item not found: issue-123");
}

#[test]
fn test_item_error_validation() {
    let err = ItemError::validation("title is required");
    assert!(matches!(err, ItemError::ValidationError(_)));
    assert_eq!(format!("{err}"), "Validation error: title is required");
}

#[test]
fn test_item_error_not_initialized() {
    let err = ItemError::NotInitialized;
    assert_eq!(format!("{err}"), "Project not initialized");
}

#[test]
fn test_item_error_invalid_status() {
    let err = ItemError::InvalidStatus {
        status: "invalid".to_string(),
        allowed: vec!["open".to_string(), "closed".to_string()],
    };
    let display = format!("{err}");
    assert!(display.contains("Invalid status 'invalid'"));
    assert!(display.contains("open"));
    assert!(display.contains("closed"));
}

#[test]
fn test_item_error_invalid_priority() {
    let err = ItemError::InvalidPriority {
        priority: 10,
        max: 3,
    };
    let display = format!("{err}");
    assert!(display.contains("Invalid priority 10"));
    assert!(display.contains("between 1 and 3"));
}

#[test]
fn test_item_error_already_exists() {
    let err = ItemError::AlreadyExists("abc-123".to_string());
    assert_eq!(format!("{err}"), "Item already exists: abc-123");
}

#[test]
fn test_item_error_is_deleted() {
    let err = ItemError::IsDeleted("abc-123".to_string());
    assert_eq!(format!("{err}"), "Item is deleted: abc-123");
}

#[test]
fn test_item_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = ItemError::from(io_err);
    assert!(matches!(err, ItemError::IoError(_)));
    let display = format!("{err}");
    assert!(display.contains("IO error"));
}

#[test]
fn test_item_error_custom_with_string() {
    let err = ItemError::custom(String::from("dynamic error"));
    assert_eq!(format!("{err}"), "dynamic error");
}

#[test]
fn test_item_error_not_found_with_string() {
    let err = ItemError::not_found(String::from("doc-xyz"));
    assert_eq!(format!("{err}"), "Item not found: doc-xyz");
}

#[test]
fn test_item_error_validation_with_string() {
    let err = ItemError::validation(String::from("bad input"));
    assert_eq!(format!("{err}"), "Validation error: bad input");
}
