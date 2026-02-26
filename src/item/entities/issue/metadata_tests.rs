use super::*;

#[test]
fn test_deserialize_priority_number() {
    let json =
        r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
    let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.common.priority, 1);
}

#[test]
fn test_deserialize_priority_string_high() {
    let json = r#"{"status":"open","priority":"high","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
    let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.common.priority, 1);
}

#[test]
fn test_deserialize_priority_string_medium() {
    let json = r#"{"status":"open","priority":"medium","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
    let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.common.priority, 2);
}

#[test]
fn test_deserialize_priority_string_low() {
    let json = r#"{"status":"open","priority":"low","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
    let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.common.priority, 3);
}

#[test]
fn test_serialize_priority_as_number() {
    let metadata = IssueMetadata::new(1, "open".to_string(), 2, HashMap::new());
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains(r#""priority":2"#));
}

#[test]
fn test_metadata_new() {
    let metadata = IssueMetadata::new(1, "open".to_string(), 1, HashMap::new());
    assert_eq!(metadata.common.display_number, 1);
    assert_eq!(metadata.common.status, "open");
    assert_eq!(metadata.common.priority, 1);
    assert!(!metadata.common.created_at.is_empty());
    assert!(!metadata.common.updated_at.is_empty());
}

#[test]
fn test_deserialize_legacy_without_display_number() {
    // Legacy issues without display_number should default to 0
    let json =
        r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
    let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.common.display_number, 0);
}

#[test]
fn test_serialize_display_number() {
    let metadata = IssueMetadata::new(42, "open".to_string(), 1, HashMap::new());
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains(r#""displayNumber":42"#));
}

#[test]
fn test_flatten_produces_flat_json() {
    // Verify that #[serde(flatten)] produces flat JSON, not nested under "common"
    let metadata = IssueMetadata::new(1, "open".to_string(), 2, HashMap::new());
    let json = serde_json::to_string(&metadata).unwrap();
    // Should NOT contain "common" as a key
    assert!(!json.contains(r#""common""#));
    // Should contain flattened fields directly
    assert!(json.contains(r#""displayNumber""#));
    assert!(json.contains(r#""status""#));
    assert!(json.contains(r#""priority""#));
}
