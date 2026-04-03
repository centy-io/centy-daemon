use super::super::build::build_config;
use crate::config::item_type_config::{ItemTypeConfig, ItemTypeFeatures};
use crate::server::convert_entity::config_to_proto;
use crate::server::proto::CreateItemTypeRequest;
use mdstore::IdStrategy;
#[test]
fn test_is_valid_plural() {
    let valid = |s: &str| !s.is_empty() && slug::slugify(s) == s;
    assert!(valid("bugs"));
    assert!(valid("user-stories"));
    assert!(valid("epics123"));
    assert!(!valid(""));
    assert!(!valid("Bugs"));
    assert!(!valid("bug_reports"));
    assert!(!valid("-bugs"));
    assert!(!valid("bugs-"));
    assert!(!valid("my bugs"));
}
#[test]
fn test_config_to_proto_roundtrip() {
    let config = ItemTypeConfig {
        name: "Bug".to_string(),
        icon: Some("bug".to_string()),
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures {
            display_number: true,
            priority: true,
            soft_delete: true,
            assets: false,
            org_sync: false,
            move_item: true,
            duplicate: true,
        },
        statuses: vec!["open".to_string(), "closed".to_string()],
        priority_levels: Some(3),
        custom_fields: vec![],
        template: Some("bug.md".to_string()),
        listed: true,
    };
    let proto = config_to_proto("bugs", &config);
    assert_eq!(proto.name, "Bug");
    assert_eq!(proto.plural, "bugs");
    assert_eq!(proto.identifier, "uuid");
    assert_eq!(proto.statuses, vec!["open", "closed"]);
    assert_eq!(proto.default_status, "open");
    assert_eq!(proto.priority_levels, 3);
    assert_eq!(proto.icon, "bug");
    assert_eq!(proto.template, "bug.md");
    let f = proto.features.unwrap();
    assert!(f.display_number);
    assert!(f.status);
    assert!(f.priority);
    assert!(f.soft_delete);
    assert!(!f.assets);
    assert!(f.r#move);
    assert!(f.duplicate);
}
#[test]
fn test_build_config_basic() {
    let req = CreateItemTypeRequest {
        project_path: String::new(),
        name: "Task".into(),
        plural: "tasks".into(),
        identifier: "uuid".into(),
        features: None,
        statuses: vec![],
        default_status: String::new(),
        priority_levels: 0,
        custom_fields: vec![],
    };
    let config = build_config(req);
    assert_eq!(config.name, "Task");
    assert!(matches!(config.identifier, IdStrategy::Uuid));
}
