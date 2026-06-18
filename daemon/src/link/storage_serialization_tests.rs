//! Tests for link/storage/serialization.rs covering all branches.
#![allow(clippy::unwrap_used)]

use super::serialization::{create_link_fields, item_to_link_record, update_link_fields};
use crate::link::TargetType;
use std::collections::HashMap;

fn make_item(custom_fields: HashMap<String, serde_json::Value>) -> mdstore::Item {
    mdstore::Item {
        id: "test-id".to_string(),
        title: String::new(),
        body: String::new(),
        frontmatter: mdstore::Frontmatter {
            display_number: None,
            status: None,
            priority: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            deleted_at: None,
            tags: None,
            custom_fields,
        },
        comment: None,
    }
}

fn full_fields() -> HashMap<String, serde_json::Value> {
    use serde_json::json;
    let mut m = HashMap::new();
    m.insert("sourceId".to_string(), json!("src"));
    m.insert("sourceType".to_string(), json!("issue"));
    m.insert("targetId".to_string(), json!("tgt"));
    m.insert("targetType".to_string(), json!("doc"));
    m.insert("linkType".to_string(), json!("blocks"));
    m
}

// ─── item_to_link_record ─────────────────────────────────────────────────────

#[test]
fn test_item_to_link_record_happy_path() {
    let record = item_to_link_record(make_item(full_fields())).unwrap();
    assert_eq!(record.id, "test-id");
    assert_eq!(record.source_id, "src");
    assert_eq!(record.source_type, TargetType::issue());
    assert_eq!(record.target_id, "tgt");
    assert_eq!(record.target_type, TargetType::new("doc"));
    assert_eq!(record.link_type, "blocks");
}

#[test]
fn test_item_to_link_record_missing_source_id_returns_none() {
    let mut fields = full_fields();
    fields.remove("sourceId");
    assert!(item_to_link_record(make_item(fields)).is_none());
}

#[test]
fn test_item_to_link_record_missing_source_type_returns_none() {
    let mut fields = full_fields();
    fields.remove("sourceType");
    assert!(item_to_link_record(make_item(fields)).is_none());
}

#[test]
fn test_item_to_link_record_missing_target_id_returns_none() {
    let mut fields = full_fields();
    fields.remove("targetId");
    assert!(item_to_link_record(make_item(fields)).is_none());
}

#[test]
fn test_item_to_link_record_missing_target_type_returns_none() {
    let mut fields = full_fields();
    fields.remove("targetType");
    assert!(item_to_link_record(make_item(fields)).is_none());
}

#[test]
fn test_item_to_link_record_missing_link_type_returns_none() {
    let mut fields = full_fields();
    fields.remove("linkType");
    assert!(item_to_link_record(make_item(fields)).is_none());
}

// ─── create_link_fields ──────────────────────────────────────────────────────

#[test]
fn test_create_link_fields_contains_all_keys() {
    let fields = create_link_fields(
        "src",
        &TargetType::issue(),
        "tgt",
        &TargetType::new("doc"),
        "blocks",
    );
    assert_eq!(fields["sourceId"], serde_json::json!("src"));
    assert_eq!(fields["sourceType"], serde_json::json!("issue"));
    assert_eq!(fields["targetId"], serde_json::json!("tgt"));
    assert_eq!(fields["targetType"], serde_json::json!("doc"));
    assert_eq!(fields["linkType"], serde_json::json!("blocks"));
}

// ─── update_link_fields ──────────────────────────────────────────────────────

#[test]
fn test_update_link_fields_contains_link_type() {
    let fields = update_link_fields("relates-to");
    assert_eq!(fields["linkType"], serde_json::json!("relates-to"));
    assert_eq!(fields.len(), 1);
}
