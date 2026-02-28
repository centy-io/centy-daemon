use super::*;

#[test]
fn test_target_type_eq() {
    assert_eq!(TargetType::issue(), TargetType::issue());
    assert_ne!(TargetType::issue(), TargetType::new("doc"));
}

#[test]
fn test_link_new_creates_timestamp() {
    let link = Link::new(
        "target-1".to_string(),
        TargetType::new("doc"),
        "relates-to".to_string(),
    );
    assert_eq!(link.target_id, "target-1");
    assert_eq!(link.target_type, TargetType::new("doc"));
    assert_eq!(link.link_type, "relates-to");
    assert!(!link.created_at.is_empty());
}

#[test]
fn test_link_deserialization() {
    let json = r#"{
        "targetId": "abc-123",
        "targetType": "issue",
        "linkType": "blocks",
        "createdAt": "2024-01-01T00:00:00Z"
    }"#;

    let link: Link = serde_json::from_str(json).unwrap();
    assert_eq!(link.target_id, "abc-123");
    assert_eq!(link.target_type, TargetType::issue());
    assert_eq!(link.link_type, "blocks");
    assert_eq!(link.created_at, "2024-01-01T00:00:00Z");
}

#[test]
fn test_custom_link_type_definition_serialization() {
    let def = CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        inverse: "dependency-of".to_string(),
        description: Some("Dependency relationship".to_string()),
    };

    let json = serde_json::to_string(&def).unwrap();
    let deserialized: CustomLinkTypeDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "depends-on");
    assert_eq!(deserialized.inverse, "dependency-of");
    assert_eq!(
        deserialized.description,
        Some("Dependency relationship".to_string())
    );
}

#[test]
fn test_custom_link_type_definition_without_description() {
    let def = CustomLinkTypeDefinition {
        name: "test".to_string(),
        inverse: "test-inverse".to_string(),
        description: None,
    };

    let json = serde_json::to_string(&def).unwrap();
    assert!(!json.contains("description"));
}

#[test]
fn test_builtin_link_types_count() {
    assert_eq!(BUILTIN_LINK_TYPES.len(), 8); // 4 pairs = 8 entries
}
