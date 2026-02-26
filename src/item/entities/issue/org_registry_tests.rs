use super::*;

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
