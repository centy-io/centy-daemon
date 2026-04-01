#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

#[test]
fn test_org_issue_registry_error_io_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let err = OrgIssueRegistryError::IoError(io_err);
    let display = format!("{err}");
    assert!(display.contains("IO error"));
    assert!(display.contains("file missing"));
}

#[test]
fn test_org_issue_registry_error_json_display() {
    let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let err = OrgIssueRegistryError::JsonError(json_err);
    let display = format!("{err}");
    assert!(display.contains("JSON error"));
}

#[test]
fn test_org_issue_registry_error_home_dir_not_found_display() {
    let err = OrgIssueRegistryError::HomeDirNotFound;
    let display = format!("{err}");
    assert!(display.contains("home directory"));
}

#[test]
fn test_org_issue_registry_default() {
    let registry = OrgIssueRegistry::default();
    assert!(registry.next_display_number.is_empty());
    assert!(!registry.updated_at.is_empty());
}

#[test]
fn test_org_issue_registry_new() {
    let registry = OrgIssueRegistry::new();
    assert!(registry.next_display_number.is_empty());
    assert!(!registry.updated_at.is_empty());
}

#[test]
fn test_org_issue_registry_serialization() {
    let mut registry = OrgIssueRegistry::new();
    registry.next_display_number.insert("my-org".to_string(), 5);

    let json = serde_json::to_string(&registry).unwrap();
    assert!(json.contains("\"nextDisplayNumber\""));
    assert!(json.contains("\"my-org\":5"));
    assert!(json.contains("\"updatedAt\""));
}

#[test]
fn test_org_issue_registry_deserialization() {
    let json = r#"{"nextDisplayNumber":{"test-org":10},"updatedAt":"2025-01-01T00:00:00Z"}"#;
    let registry: OrgIssueRegistry = serde_json::from_str(json).unwrap();

    assert_eq!(registry.next_display_number.get("test-org"), Some(&10));
    assert_eq!(registry.updated_at, "2025-01-01T00:00:00Z");
}
