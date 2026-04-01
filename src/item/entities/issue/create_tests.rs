#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::helpers::{build_custom_fields, build_issue_for_sync, resolve_priority};
use super::types::CreateIssueOptions;
use crate::config::CentyConfig;
use crate::item::entities::issue::metadata::IssueMetadata;
use std::collections::HashMap;

// --- resolve_priority tests ---

#[test]
fn test_resolve_priority_none_no_config() {
    // No priority provided, no config -> default_priority(3) == 2
    let result = resolve_priority(None, None, 3).unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_resolve_priority_with_explicit_value() {
    let result = resolve_priority(Some(1), None, 3).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_resolve_priority_invalid_exceeds_levels() {
    let result = resolve_priority(Some(5), None, 3);
    assert!(result.is_err());
}

#[test]
fn test_resolve_priority_from_config_default() {
    let mut config = CentyConfig::default();
    config
        .defaults
        .insert("priority".to_string(), "1".to_string());
    let result = resolve_priority(None, Some(&config), 3).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_resolve_priority_config_default_invalid_string_falls_back() {
    let mut config = CentyConfig::default();
    config
        .defaults
        .insert("priority".to_string(), "not-a-number".to_string());
    // Invalid string can't parse, falls back to default_priority(3) == 2
    let result = resolve_priority(None, Some(&config), 3).unwrap();
    assert_eq!(result, 2);
}

// --- build_custom_fields tests ---

#[test]
fn test_build_custom_fields_no_config_no_provided() {
    let provided: HashMap<String, String> = HashMap::new();
    let fields = build_custom_fields(None, &provided);
    assert!(fields.is_empty());
}

#[test]
fn test_build_custom_fields_with_provided_fields() {
    let mut provided: HashMap<String, String> = HashMap::new();
    provided.insert("team".to_string(), "backend".to_string());
    let fields = build_custom_fields(None, &provided);
    assert_eq!(
        fields.get("team"),
        Some(&serde_json::Value::String("backend".to_string()))
    );
}

#[test]
fn test_build_custom_fields_provided_overrides_config_defaults() {
    let mut config = CentyConfig::default();
    config.custom_fields.push(mdstore::CustomFieldDef {
        name: "team".to_string(),
        field_type: "string".to_string(),
        default_value: Some("frontend".to_string()),
        required: false,
        enum_values: vec![],
    });
    let mut provided: HashMap<String, String> = HashMap::new();
    provided.insert("team".to_string(), "backend".to_string());
    let fields = build_custom_fields(Some(&config), &provided);
    // Provided value overrides config default
    assert_eq!(
        fields.get("team"),
        Some(&serde_json::Value::String("backend".to_string()))
    );
}

#[test]
fn test_build_custom_fields_config_default_without_override() {
    let mut config = CentyConfig::default();
    config.custom_fields.push(mdstore::CustomFieldDef {
        name: "env".to_string(),
        field_type: "string".to_string(),
        default_value: Some("production".to_string()),
        required: false,
        enum_values: vec![],
    });
    let provided: HashMap<String, String> = HashMap::new();
    let fields = build_custom_fields(Some(&config), &provided);
    assert_eq!(
        fields.get("env"),
        Some(&serde_json::Value::String("production".to_string()))
    );
}

#[test]
fn test_build_custom_fields_config_field_without_default_not_added() {
    let mut config = CentyConfig::default();
    config.custom_fields.push(mdstore::CustomFieldDef {
        name: "optional-field".to_string(),
        field_type: "string".to_string(),
        default_value: None,
        required: false,
        enum_values: vec![],
    });
    let provided: HashMap<String, String> = HashMap::new();
    let fields = build_custom_fields(Some(&config), &provided);
    assert!(!fields.contains_key("optional-field"));
}

// --- build_issue_for_sync tests ---

#[test]
fn test_build_issue_for_sync() {
    let options = CreateIssueOptions {
        title: "Sync Issue".to_string(),
        description: "Some description".to_string(),
        ..Default::default()
    };
    let meta = IssueMetadata::new(3, "open".to_string(), 2, HashMap::new());
    let issue = build_issue_for_sync("test-uuid", &options, 3, &meta);
    assert_eq!(issue.id, "test-uuid");
    assert_eq!(issue.title, "Sync Issue");
    assert_eq!(issue.description, "Some description");
    assert_eq!(issue.metadata.display_number, 3);
    assert_eq!(issue.metadata.status, "open");
    assert_eq!(issue.metadata.priority, 2);
    assert!(!issue.metadata.is_org_issue);
    assert!(issue.metadata.org_slug.is_none());
}

#[test]
fn test_build_issue_for_sync_org_issue() {
    let options = CreateIssueOptions {
        title: "Org Issue".to_string(),
        description: String::new(),
        is_org_issue: true,
        ..Default::default()
    };
    let meta =
        IssueMetadata::new_org_issue(5, 42, "open".to_string(), 1, "acme", HashMap::new(), false);
    let issue = build_issue_for_sync("org-uuid", &options, 5, &meta);
    assert!(issue.metadata.is_org_issue);
    assert_eq!(issue.metadata.org_slug, Some("acme".to_string()));
    assert_eq!(issue.metadata.org_display_number, Some(42));
}
