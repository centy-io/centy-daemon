#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

// --- IssueFrontmatter tests ---

#[test]
fn test_issue_frontmatter_deserialize() {
    use mdstore::parse_frontmatter;
    let yaml = "---\ndisplayNumber: 5\nstatus: open\npriority: 2\ncreatedAt: 2024-01-01T00:00:00Z\nupdatedAt: 2024-01-01T00:00:00Z\n---\n# Title\n\nBody";
    let (fm, title, _): (IssueFrontmatter, String, String) = parse_frontmatter(yaml).unwrap();
    assert_eq!(fm.display_number, 5);
    assert_eq!(fm.status, "open");
    assert_eq!(fm.priority, 2);
    assert_eq!(title, "Title");
    assert!(!fm.draft);
    assert!(fm.deleted_at.is_none());
}

#[test]
fn test_issue_frontmatter_serialize_skips_draft_when_false() {
    use mdstore::generate_frontmatter;
    let fm = IssueFrontmatter {
        display_number: 1,
        status: "open".to_string(),
        priority: 1,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        draft: false,
        deleted_at: None,
        custom_fields: std::collections::HashMap::new(),
    };
    let output = generate_frontmatter(&fm, "Test", "Body", None);
    assert!(!output.contains("draft"));
}

#[test]
fn test_issue_frontmatter_serialize_includes_draft_when_true() {
    use mdstore::generate_frontmatter;
    let fm = IssueFrontmatter {
        display_number: 1,
        status: "open".to_string(),
        priority: 1,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        draft: true,
        deleted_at: None,
        custom_fields: std::collections::HashMap::new(),
    };
    let output = generate_frontmatter(&fm, "Test", "Body", None);
    assert!(output.contains("draft: true"));
}
