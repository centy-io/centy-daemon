//! Additional tests for `create/helpers.rs` covering `resolve_org_info` and additional branches.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::helpers::{
    build_custom_fields, build_issue_for_sync, resolve_org_info, resolve_priority,
};
use super::types::{CreateIssueOptions, IssueError};
use crate::config::CentyConfig;
use crate::item::entities::issue::metadata::IssueMetadata;
use std::collections::HashMap;

// --- resolve_org_info tests ---

#[tokio::test]
async fn test_resolve_org_info_not_org_issue() {
    // is_org_issue=false => always returns (None, None) without any IO
    let temp = tempfile::tempdir().unwrap();
    let result = resolve_org_info(temp.path(), false).await.unwrap();
    assert_eq!(result, (None, None));
}

#[tokio::test]
async fn test_resolve_org_info_is_org_issue_no_project_info() {
    // is_org_issue=true but project isn't in registry => project_info=None => NoOrganization
    // Use a freshly created temp path that is guaranteed not to be in the registry
    let temp = tempfile::tempdir().unwrap();
    let result = resolve_org_info(temp.path(), true).await;
    // project_info returns None for unknown path, then organization_slug is None => NoOrganization
    assert!(matches!(result, Err(IssueError::NoOrganization)));
}

// --- resolve_priority edge cases ---

#[test]
fn test_resolve_priority_priority_zero_invalid() {
    let result = resolve_priority(Some(0), None, 3);
    assert!(result.is_err());
}

#[test]
fn test_resolve_priority_at_max_level() {
    let result = resolve_priority(Some(3), None, 3).unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_resolve_priority_exceeds_max_level() {
    let result = resolve_priority(Some(4), None, 3);
    assert!(result.is_err());
}

#[test]
fn test_resolve_priority_levels_1_default_is_1() {
    let result = resolve_priority(None, None, 1).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_resolve_priority_levels_2_default_is_1() {
    let result = resolve_priority(None, None, 2).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_resolve_priority_levels_4_default_is_2() {
    let result = resolve_priority(None, None, 4).unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_resolve_priority_config_no_priority_key() {
    // Config has defaults but no "priority" key -> uses default_priority
    let config = CentyConfig::default();
    let result = resolve_priority(None, Some(&config), 3).unwrap();
    assert_eq!(result, 2);
}

// --- build_custom_fields edge cases ---

#[test]
fn test_build_custom_fields_multiple_config_fields() {
    let mut config = CentyConfig::default();
    config.custom_fields.push(mdstore::CustomFieldDef {
        name: "field1".to_string(),
        field_type: "string".to_string(),
        default_value: Some("default1".to_string()),
        required: false,
        enum_values: vec![],
    });
    config.custom_fields.push(mdstore::CustomFieldDef {
        name: "field2".to_string(),
        field_type: "string".to_string(),
        default_value: Some("default2".to_string()),
        required: false,
        enum_values: vec![],
    });
    let provided: HashMap<String, String> = HashMap::new();
    let fields = build_custom_fields(Some(&config), &provided);
    assert_eq!(fields.len(), 2);
    assert_eq!(
        fields.get("field1"),
        Some(&serde_json::Value::String("default1".to_string()))
    );
    assert_eq!(
        fields.get("field2"),
        Some(&serde_json::Value::String("default2".to_string()))
    );
}

#[test]
fn test_build_custom_fields_provided_added_when_no_config() {
    let mut provided = HashMap::new();
    provided.insert("custom_key".to_string(), "custom_value".to_string());
    let fields = build_custom_fields(None, &provided);
    assert_eq!(
        fields.get("custom_key"),
        Some(&serde_json::Value::String("custom_value".to_string()))
    );
}

// --- build_issue_for_sync edge cases ---

#[test]
fn test_build_issue_for_sync_no_description() {
    let options = CreateIssueOptions {
        title: "No Desc".to_string(),
        description: String::new(),
        ..Default::default()
    };
    let meta = IssueMetadata::new(1, "open".to_string(), 1, HashMap::new());
    let issue = build_issue_for_sync("uuid-no-desc", &options, 1, &meta);
    assert_eq!(issue.id, "uuid-no-desc");
    assert_eq!(issue.description, "");
}

#[test]
fn test_build_issue_for_sync_with_deleted_at() {
    let options = CreateIssueOptions {
        title: "Deleted Issue".to_string(),
        description: String::new(),
        ..Default::default()
    };
    let mut meta = IssueMetadata::new(2, "closed".to_string(), 3, HashMap::new());
    meta.deleted_at = Some("2024-01-01T00:00:00.000000+00:00".to_string());
    let issue = build_issue_for_sync("deleted-uuid", &options, 2, &meta);
    assert_eq!(
        issue.metadata.deleted_at,
        Some("2024-01-01T00:00:00.000000+00:00".to_string())
    );
}

#[test]
fn test_build_issue_for_sync_custom_fields_from_options() {
    let mut custom = HashMap::new();
    custom.insert("env".to_string(), "staging".to_string());
    let options = CreateIssueOptions {
        title: "Custom Fields Issue".to_string(),
        description: String::new(),
        custom_fields: custom,
        ..Default::default()
    };
    let meta = IssueMetadata::new(1, "open".to_string(), 1, HashMap::new());
    let issue = build_issue_for_sync("cf-uuid", &options, 1, &meta);
    assert_eq!(
        issue.metadata.custom_fields.get("env"),
        Some(&"staging".to_string())
    );
}

#[test]
fn test_build_issue_for_sync_draft_field() {
    let options = CreateIssueOptions {
        title: "Draft".to_string(),
        description: String::new(),
        draft: Some(true),
        ..Default::default()
    };
    let meta = IssueMetadata::new_draft(1, "open".to_string(), 1, HashMap::new(), true);
    let issue = build_issue_for_sync("draft-uuid", &options, 1, &meta);
    assert!(issue.metadata.draft);
}
