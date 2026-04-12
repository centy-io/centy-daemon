use super::*;
#[test]
fn test_organization_serialization() {
    let org = Organization {
        name: "Acme Corp".to_string(),
        description: Some("Our company".to_string()),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-06-15T12:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&org).expect("Should serialize");
    let deserialized: Organization = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.name, "Acme Corp");
    assert_eq!(deserialized.description, Some("Our company".to_string()));
}
#[test]
fn test_organization_without_description() {
    let org = Organization {
        name: "Test".to_string(),
        description: None,
        created_at: "2024-01-01".to_string(),
        updated_at: "2024-01-01".to_string(),
    };
    let json = serde_json::to_string(&org).expect("Should serialize");
    assert!(!json.contains("description"));
}
#[test]
fn test_organization_camel_case() {
    let org = Organization {
        name: "Test".to_string(),
        description: None,
        created_at: "2024-01-01".to_string(),
        updated_at: "2024-01-01".to_string(),
    };
    let json = serde_json::to_string(&org).expect("Should serialize");
    assert!(json.contains("createdAt"));
    assert!(json.contains("updatedAt"));
}
#[test]
fn test_project_organization_serialization() {
    let po = ProjectOrganization {
        slug: "acme".to_string(),
        name: "Acme Corp".to_string(),
        description: Some("desc".to_string()),
    };
    let json = serde_json::to_string(&po).expect("Should serialize");
    let deserialized: ProjectOrganization =
        serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.slug, "acme");
    assert_eq!(deserialized.name, "Acme Corp");
}
#[test]
fn test_tracked_project_serialization() {
    let tp = TrackedProject {
        first_accessed: "2024-01-01".to_string(),
        last_accessed: "2024-06-15".to_string(),
        is_favorite: true,
        is_archived: false,
        organization_slug: Some("acme".to_string()),
        user_title: Some("My Project".to_string()),
    };
    let json = serde_json::to_string(&tp).expect("Should serialize");
    let deserialized: TrackedProject = serde_json::from_str(&json).expect("Should deserialize");
    assert!(deserialized.is_favorite);
    assert!(!deserialized.is_archived);
    assert_eq!(deserialized.organization_slug, Some("acme".to_string()));
    assert_eq!(deserialized.user_title, Some("My Project".to_string()));
}
#[test]
fn test_tracked_project_defaults() {
    let json = r#"{"firstAccessed":"2024-01-01","lastAccessed":"2024-01-01"}"#;
    let tp: TrackedProject = serde_json::from_str(json).expect("Should deserialize");
    assert!(!tp.is_favorite);
    assert!(!tp.is_archived);
    assert!(tp.organization_slug.is_none());
    assert!(tp.user_title.is_none());
}
#[test]
fn test_tracked_project_skip_serializing_none() {
    let tp = TrackedProject {
        first_accessed: "2024-01-01".to_string(),
        last_accessed: "2024-01-01".to_string(),
        is_favorite: false,
        is_archived: false,
        organization_slug: None,
        user_title: None,
    };
    let json = serde_json::to_string(&tp).expect("Should serialize");
    assert!(!json.contains("organizationSlug"));
    assert!(!json.contains("userTitle"));
}
