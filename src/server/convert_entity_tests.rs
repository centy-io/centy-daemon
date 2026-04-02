use super::*;
use crate::config::item_type_config::{ItemTypeConfig, ItemTypeFeatures};
use mdstore::{CustomFieldDef, IdStrategy};
use std::collections::HashMap;

fn make_item(
    display_number: Option<u32>,
    status: Option<&str>,
    priority: Option<u32>,
    deleted_at: Option<&str>,
    tags: Option<Vec<String>>,
    custom_fields: HashMap<String, serde_json::Value>,
) -> mdstore::Item {
    mdstore::Item {
        id: "test-uuid".to_string(),
        title: "Test Item".to_string(),
        body: "Body content".to_string(),
        frontmatter: mdstore::Frontmatter {
            display_number,
            status: status.map(str::to_string),
            priority,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
            deleted_at: deleted_at.map(str::to_string),
            tags,
            custom_fields,
        },
        comment: None,
    }
}

#[test]
fn test_generic_item_to_proto_basic() {
    let item = make_item(Some(42), Some("open"), Some(2), None, None, HashMap::new());
    let proto = generic_item_to_proto(&item, "issue");
    assert_eq!(proto.id, "test-uuid");
    assert_eq!(proto.item_type, "issue");
    assert_eq!(proto.title, "Test Item");
    assert_eq!(proto.body, "Body content");
    let meta = proto.metadata.unwrap();
    assert_eq!(meta.display_number, 42);
    assert_eq!(meta.status, "open");
    assert_eq!(meta.priority, 2);
    assert_eq!(meta.deleted_at, "");
}

#[test]
fn test_generic_item_to_proto_defaults_when_none() {
    // display_number=None → 0, status=None → "", priority=None → 0
    let item = make_item(None, None, None, None, None, HashMap::new());
    let proto = generic_item_to_proto(&item, "doc");
    let meta = proto.metadata.unwrap();
    assert_eq!(meta.display_number, 0);
    assert_eq!(meta.status, "");
    assert_eq!(meta.priority, 0);
    assert_eq!(meta.deleted_at, "");
}

#[test]
fn test_generic_item_to_proto_with_deleted_at() {
    let item = make_item(
        None,
        None,
        None,
        Some("2024-06-01T00:00:00Z"),
        None,
        HashMap::new(),
    );
    let proto = generic_item_to_proto(&item, "issue");
    let meta = proto.metadata.unwrap();
    assert_eq!(meta.deleted_at, "2024-06-01T00:00:00Z");
}

#[test]
fn test_generic_item_to_proto_with_tags() {
    let item = make_item(
        None,
        None,
        None,
        None,
        Some(vec!["bug".to_string(), "urgent".to_string()]),
        HashMap::new(),
    );
    let proto = generic_item_to_proto(&item, "issue");
    let meta = proto.metadata.unwrap();
    assert_eq!(meta.tags, vec!["bug", "urgent"]);
}

#[test]
fn test_generic_item_to_proto_with_custom_fields() {
    let custom_fields = HashMap::from([("env".to_string(), serde_json::json!("prod"))]);
    let item = make_item(None, None, None, None, None, custom_fields);
    let proto = generic_item_to_proto(&item, "issue");
    let meta = proto.metadata.unwrap();
    assert_eq!(
        meta.custom_fields.get("env").map(String::as_str),
        Some("\"prod\"")
    );
}

#[test]
fn test_user_to_proto_with_email() {
    let user = crate::user::User {
        id: "alice".to_string(),
        name: "Alice".to_string(),
        email: Some("alice@example.com".to_string()),
        git_usernames: vec!["alice-git".to_string()],
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
        deleted_at: None,
    };
    let proto = user_to_proto(&user);
    assert_eq!(proto.id, "alice");
    assert_eq!(proto.name, "Alice");
    assert_eq!(proto.email, "alice@example.com");
    assert_eq!(proto.git_usernames, vec!["alice-git"]);
    assert_eq!(proto.deleted_at, "");
}

#[test]
fn test_user_to_proto_without_email_and_deleted() {
    let user = crate::user::User {
        id: "bob".to_string(),
        name: "Bob".to_string(),
        email: None,
        git_usernames: vec![],
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
        deleted_at: Some("2024-06-01T00:00:00Z".to_string()),
    };
    let proto = user_to_proto(&user);
    assert_eq!(proto.email, "");
    assert_eq!(proto.deleted_at, "2024-06-01T00:00:00Z");
}

#[test]
fn test_user_to_generic_item_proto_active() {
    let user = crate::user::User {
        id: "alice".to_string(),
        name: "Alice".to_string(),
        email: None,
        git_usernames: vec![],
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
        deleted_at: None,
    };
    let proto = user_to_generic_item_proto(&user);
    assert_eq!(proto.id, "alice");
    assert_eq!(proto.item_type, "user");
    assert_eq!(proto.title, "Alice");
    assert_eq!(proto.body, "");
    let meta = proto.metadata.unwrap();
    assert_eq!(meta.status, "active");
    assert_eq!(meta.display_number, 0);
    assert_eq!(meta.priority, 0);
    assert_eq!(meta.deleted_at, "");
}

#[test]
fn test_user_to_generic_item_proto_deleted() {
    let user = crate::user::User {
        id: "bob".to_string(),
        name: "Bob".to_string(),
        email: None,
        git_usernames: vec![],
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
        deleted_at: Some("2024-06-01T00:00:00Z".to_string()),
    };
    let proto = user_to_generic_item_proto(&user);
    let meta = proto.metadata.unwrap();
    assert_eq!(meta.status, "deleted");
    assert_eq!(meta.deleted_at, "2024-06-01T00:00:00Z");
}

#[test]
fn test_config_to_proto_minimal() {
    let config = ItemTypeConfig {
        name: "Issue".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures {
            display_number: true,
            priority: true,
            soft_delete: true,
            assets: false,
            org_sync: false,
            move_item: true,
            duplicate: false,
        },
        statuses: vec!["open".to_string(), "closed".to_string()],
        priority_levels: Some(3),
        custom_fields: vec![],
        template: None,
        listed: true,
    };
    let proto = config_to_proto("issues", &config);
    assert_eq!(proto.name, "Issue");
    assert_eq!(proto.plural, "issues");
    assert_eq!(proto.identifier, "uuid");
    assert_eq!(proto.default_status, "open");
    assert_eq!(proto.priority_levels, 3);
    assert_eq!(proto.icon, "");
    assert_eq!(proto.template, "");
    let feats = proto.features.unwrap();
    assert!(feats.display_number);
    assert!(feats.status); // statuses non-empty → true
    assert!(feats.priority);
    assert!(feats.soft_delete);
    assert!(!feats.assets);
}

#[test]
fn test_config_to_proto_no_statuses() {
    let config = ItemTypeConfig {
        name: "Doc".to_string(),
        icon: Some("document".to_string()),
        identifier: IdStrategy::Slug,
        features: ItemTypeFeatures {
            display_number: false,
            priority: false,
            soft_delete: false,
            assets: false,
            org_sync: false,
            move_item: false,
            duplicate: false,
        },
        statuses: vec![],
        priority_levels: None,
        custom_fields: vec![],
        template: Some("default.hbs".to_string()),
        listed: true,
    };
    let proto = config_to_proto("docs", &config);
    assert_eq!(proto.default_status, "");
    assert_eq!(proto.priority_levels, 0);
    assert_eq!(proto.icon, "document");
    assert_eq!(proto.template, "default.hbs");
    let feats = proto.features.unwrap();
    assert!(!feats.status); // empty statuses → false
}

#[test]
fn test_config_to_proto_with_custom_fields() {
    let config = ItemTypeConfig {
        name: "Epic".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: vec![],
        priority_levels: None,
        custom_fields: vec![
            CustomFieldDef {
                name: "team".to_string(),
                field_type: "text".to_string(),
                required: true,
                default_value: Some("platform".to_string()),
                enum_values: vec![],
            },
            CustomFieldDef {
                name: "env".to_string(),
                field_type: "enum".to_string(),
                required: false,
                default_value: None,
                enum_values: vec!["prod".to_string(), "staging".to_string()],
            },
        ],
        template: None,
        listed: true,
    };
    let proto = config_to_proto("epics", &config);
    assert_eq!(proto.custom_fields.len(), 2);
    assert_eq!(proto.custom_fields[0].name, "team");
    assert_eq!(proto.custom_fields[0].field_type, "text");
    assert!(proto.custom_fields[0].required);
    assert_eq!(proto.custom_fields[0].default_value, "platform");
    assert_eq!(proto.custom_fields[1].name, "env");
    assert!(!proto.custom_fields[1].required);
    assert_eq!(proto.custom_fields[1].default_value, "");
    assert_eq!(proto.custom_fields[1].enum_values, vec!["prod", "staging"]);
}
