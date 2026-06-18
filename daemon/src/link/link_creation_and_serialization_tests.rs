use super::*;

#[test]
fn test_target_type_eq() {
    assert_eq!(TargetType::issue(), TargetType::issue());
    assert_ne!(TargetType::issue(), TargetType::new("doc"));
}

#[test]
fn test_custom_link_type_definition_serialization() {
    let def = CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        description: Some("Dependency relationship".to_string()),
    };

    let json = serde_json::to_string(&def).unwrap();
    let deserialized: CustomLinkTypeDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "depends-on");
    assert_eq!(
        deserialized.description,
        Some("Dependency relationship".to_string())
    );
}

#[test]
fn test_custom_link_type_definition_without_description() {
    let def = CustomLinkTypeDefinition {
        name: "test".to_string(),
        description: None,
    };

    let json = serde_json::to_string(&def).unwrap();
    assert!(!json.contains("description"));
}

#[test]
fn test_builtin_link_types_count() {
    assert_eq!(BUILTIN_LINK_TYPES.len(), 8);
}

#[test]
fn test_link_record_source_view() {
    let record = LinkRecord {
        id: "link-uuid".to_string(),
        source_id: "src-id".to_string(),
        source_type: TargetType::issue(),
        target_id: "tgt-id".to_string(),
        target_type: TargetType::new("doc"),
        link_type: "blocks".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let view = record.source_view();
    assert_eq!(view.id, "link-uuid");
    assert_eq!(view.target_id, "tgt-id");
    assert_eq!(view.target_type, TargetType::new("doc"));
    assert_eq!(view.link_type, "blocks");
    assert_eq!(view.direction, LinkDirection::Source);
}

#[test]
fn test_link_record_target_view() {
    let record = LinkRecord {
        id: "link-uuid".to_string(),
        source_id: "src-id".to_string(),
        source_type: TargetType::issue(),
        target_id: "tgt-id".to_string(),
        target_type: TargetType::new("doc"),
        link_type: "blocks".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let view = record.target_view();
    assert_eq!(view.id, "link-uuid");
    // From target's POV, the "target" is actually the source entity.
    assert_eq!(view.target_id, "src-id");
    assert_eq!(view.target_type, TargetType::issue());
    assert_eq!(view.link_type, "blocks");
    assert_eq!(view.direction, LinkDirection::Target);
}
