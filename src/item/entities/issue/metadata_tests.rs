#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

// --- IssueFrontmatter tests ---

#[test]
fn test_issue_frontmatter_from_metadata_basic() {
    let meta = IssueMetadata::new(5, "open".to_string(), 2, std::collections::HashMap::new());
    let custom: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("team".to_string(), "backend".to_string());
        m
    };
    let fm = IssueFrontmatter::from_metadata(&meta, custom);
    assert_eq!(fm.display_number, 5);
    assert_eq!(fm.status, "open");
    assert_eq!(fm.priority, 2);
    assert!(!fm.draft);
    assert!(fm.deleted_at.is_none());
    assert!(!fm.is_org_issue);
    assert!(fm.org_slug.is_none());
    assert!(fm.org_display_number.is_none());
    assert_eq!(
        fm.custom_fields.get("team").map(String::as_str),
        Some("backend")
    );
}

#[test]
fn test_issue_frontmatter_from_metadata_org_issue() {
    let meta = IssueMetadata::new_org_issue(
        3,
        10,
        "open".to_string(),
        1,
        "my-org",
        HashMap::new(),
        false,
    );
    let fm = IssueFrontmatter::from_metadata(&meta, HashMap::new());
    assert!(fm.is_org_issue);
    assert_eq!(fm.org_slug, Some("my-org".to_string()));
    assert_eq!(fm.org_display_number, Some(10));
}

#[test]
fn test_issue_frontmatter_to_metadata_round_trip() {
    let meta = IssueMetadata::new(7, "closed".to_string(), 3, HashMap::new());
    let fm = IssueFrontmatter::from_metadata(&meta, HashMap::new());
    let restored = fm.to_metadata();
    assert_eq!(restored.common.display_number, 7);
    assert_eq!(restored.common.status, "closed");
    assert_eq!(restored.common.priority, 3);
    assert!(!restored.draft);
    assert!(restored.deleted_at.is_none());
    assert!(!restored.is_org_issue);
}

#[test]
fn test_issue_frontmatter_to_metadata_with_custom_fields() {
    let meta = IssueMetadata::new(1, "open".to_string(), 1, HashMap::new());
    let mut custom: HashMap<String, String> = HashMap::new();
    custom.insert("sprint".to_string(), "12".to_string());
    let fm = IssueFrontmatter::from_metadata(&meta, custom);
    let restored = fm.to_metadata();
    let val = restored.common.custom_fields.get("sprint");
    assert!(matches!(val, Some(serde_json::Value::String(s)) if s == "12"));
}

// --- IssueMetadata constructor tests ---

#[test]
fn test_issue_metadata_new_draft() {
    let meta = IssueMetadata::new_draft(2, "open".to_string(), 1, HashMap::new(), true);
    assert_eq!(meta.common.display_number, 2);
    assert!(meta.draft);
    assert!(!meta.is_org_issue);
    assert!(meta.deleted_at.is_none());
}

#[test]
fn test_issue_metadata_new_draft_false() {
    let meta = IssueMetadata::new_draft(1, "open".to_string(), 2, HashMap::new(), false);
    assert!(!meta.draft);
}

#[test]
fn test_issue_metadata_new_org_issue() {
    let meta = IssueMetadata::new_org_issue(
        4,
        20,
        "in-progress".to_string(),
        2,
        "acme-corp",
        HashMap::new(),
        false,
    );
    assert_eq!(meta.common.display_number, 4);
    assert_eq!(meta.org_display_number, Some(20));
    assert_eq!(meta.org_slug, Some("acme-corp".to_string()));
    assert!(meta.is_org_issue);
    assert!(!meta.draft);
    assert!(meta.deleted_at.is_none());
}

#[test]
fn test_deserialize_priority_number() {
    let json =
        r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
    let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(metadata.common.priority, 1);
}

#[test]
fn test_deserialize_priority_string_high() {
    let json =
        r#"{"status":"open","priority":"high","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
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
    let json =
        r#"{"status":"open","priority":"low","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
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
