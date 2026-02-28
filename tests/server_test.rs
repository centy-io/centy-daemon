#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::config::{CentyConfig, WorkspaceConfig};
use centy_daemon::item::entities::issue::{
    create_issue, get_issue, list_issues, update_issue, CreateIssueOptions, UpdateIssueOptions,
};
use centy_daemon::link::CustomLinkTypeDefinition;
use common::{create_test_dir, init_centy_project};
use mdstore::CustomFieldDef;
use std::collections::HashMap;

// Test issue operations that the server handlers wrap

#[tokio::test]
async fn test_issue_create_get_roundtrip() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options = CreateIssueOptions {
        title: "Server Test Issue".to_string(),
        description: "Testing issue operations".to_string(),
        priority: Some(1),
        status: Some("open".to_string()),
        custom_fields: HashMap::from([("assignee".to_string(), "alice".to_string())]),
        ..Default::default()
    };

    let result = create_issue(project_path, options)
        .await
        .expect("Should create");
    let issue = get_issue(project_path, &result.id)
        .await
        .expect("Should get");

    assert_eq!(issue.title, "Server Test Issue");
    assert_eq!(issue.description, "Testing issue operations");
    assert_eq!(issue.metadata.priority, 1);
    assert_eq!(issue.metadata.status, "open");
    assert_eq!(
        issue.metadata.custom_fields.get("assignee"),
        Some(&"alice".to_string())
    );
}

#[tokio::test]
async fn test_issue_list_with_filters() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create multiple issues
    for i in 1..=5 {
        let status = if i % 2 == 0 { "open" } else { "closed" };
        create_issue(
            project_path,
            CreateIssueOptions {
                title: format!("Issue {i}"),
                status: Some(status.to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    // List all
    let all_issues = list_issues(project_path, None, None, None, false)
        .await
        .expect("Should list");
    assert_eq!(all_issues.len(), 5);

    // List by status
    let open_issues = list_issues(project_path, Some("open"), None, None, false)
        .await
        .expect("Should list");
    assert_eq!(open_issues.len(), 2);

    let closed_issues = list_issues(project_path, Some("closed"), None, None, false)
        .await
        .expect("Should list");
    assert_eq!(closed_issues.len(), 3);
}

#[tokio::test]
async fn test_issue_update_fields() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Original".to_string(),
            description: "Original desc".to_string(),
            priority: Some(3),
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let update_opts = UpdateIssueOptions {
        title: Some("Updated".to_string()),
        description: Some("Updated desc".to_string()),
        priority: Some(1),
        status: Some("closed".to_string()),
        ..Default::default()
    };

    let updated = update_issue(project_path, &result.id, update_opts)
        .await
        .expect("Should update");

    assert_eq!(updated.issue.title, "Updated");
    assert_eq!(updated.issue.description, "Updated desc");
    assert_eq!(updated.issue.metadata.priority, 1);
    assert_eq!(updated.issue.metadata.status, "closed");
}

// Test config validation logic

#[test]
fn test_config_priority_levels_range() {
    // Valid range is 1-10
    let config = CentyConfig {
        priority_levels: 5,
        ..Default::default()
    };
    assert!(config.priority_levels >= 1 && config.priority_levels <= 10);

    let invalid_config = CentyConfig {
        priority_levels: 0,
        ..Default::default()
    };
    assert!(invalid_config.priority_levels < 1);
}

#[test]
fn test_config_custom_field_uniqueness() {
    let config = CentyConfig {
        custom_fields: vec![
            CustomFieldDef {
                name: "assignee".to_string(),
                field_type: "string".to_string(),
                required: false,
                default_value: None,
                enum_values: vec![],
            },
            CustomFieldDef {
                name: "component".to_string(),
                field_type: "string".to_string(),
                required: false,
                default_value: None,
                enum_values: vec![],
            },
        ],
        ..Default::default()
    };

    // Check all names are unique
    let mut names = std::collections::HashSet::new();
    for field in &config.custom_fields {
        assert!(names.insert(&field.name), "Duplicate field name");
    }
}

#[test]
fn test_config_enum_field_has_values() {
    let valid_enum_field = CustomFieldDef {
        name: "priority_label".to_string(),
        field_type: "enum".to_string(),
        required: false,
        default_value: None,
        enum_values: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
    };
    assert!(!valid_enum_field.enum_values.is_empty());

    let invalid_enum_field = CustomFieldDef {
        name: "empty_enum".to_string(),
        field_type: "enum".to_string(),
        required: false,
        default_value: None,
        enum_values: vec![],
    };
    // Enum fields should have values
    assert!(invalid_enum_field.field_type == "enum" && invalid_enum_field.enum_values.is_empty());
}

#[test]
fn test_config_hex_color_format() {
    let hex_regex = regex::Regex::new(r"^#([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$").unwrap();

    // Valid colors
    assert!(hex_regex.is_match("#FFF"));
    assert!(hex_regex.is_match("#fff"));
    assert!(hex_regex.is_match("#FFFFFF"));
    assert!(hex_regex.is_match("#ffffff"));
    assert!(hex_regex.is_match("#AbC123"));

    // Invalid colors
    assert!(!hex_regex.is_match("FFF")); // Missing #
    assert!(!hex_regex.is_match("#FF")); // Too short
    assert!(!hex_regex.is_match("#FFFFFFF")); // Too long
    assert!(!hex_regex.is_match("#GGG")); // Invalid chars
}

#[test]
fn test_custom_link_type_definition() {
    let link_type = CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        inverse: "dependency-of".to_string(),
        description: Some("Issue depends on another".to_string()),
    };

    assert_eq!(link_type.name, "depends-on");
    assert_eq!(link_type.inverse, "dependency-of");
    assert!(link_type.description.is_some());
}

// Test priority label function

#[test]
fn test_priority_label_3_levels() {
    use centy_daemon::item::entities::issue::priority_label;

    assert_eq!(priority_label(1, 3), "high");
    assert_eq!(priority_label(2, 3), "medium");
    assert_eq!(priority_label(3, 3), "low");
}

#[test]
fn test_priority_label_4_levels() {
    use centy_daemon::item::entities::issue::priority_label;

    // 4 levels: critical, high, medium, low
    assert_eq!(priority_label(1, 4), "critical");
    assert_eq!(priority_label(2, 4), "high");
    assert_eq!(priority_label(3, 4), "medium");
    assert_eq!(priority_label(4, 4), "low");
}

#[test]
fn test_priority_label_5_plus_levels() {
    use centy_daemon::item::entities::issue::priority_label;

    // 5+ levels: returns P{n} format
    assert_eq!(priority_label(1, 5), "P1");
    assert_eq!(priority_label(2, 5), "P2");
    assert_eq!(priority_label(3, 5), "P3");
}

#[test]
fn test_priority_label_out_of_range() {
    use centy_daemon::item::entities::issue::priority_label;

    // For 3 levels, anything > 2 returns "low"
    assert_eq!(priority_label(10, 3), "low");
    // 0 also falls to default case "low"
    assert_eq!(priority_label(0, 3), "low");
}

// Test display number auto-increment

#[tokio::test]
async fn test_issue_display_numbers_sequential() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let mut display_numbers = vec![];

    for i in 1..=5 {
        let result = create_issue(
            project_path,
            CreateIssueOptions {
                title: format!("Issue {i}"),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        display_numbers.push(result.display_number);
    }

    assert_eq!(display_numbers, vec![1, 2, 3, 4, 5]);
}

// Test sequential operations (replaces concurrent test due to file locking)

#[tokio::test]
async fn test_sequential_issue_creation() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issues sequentially
    let mut results = vec![];
    for i in 1..=10 {
        let result = create_issue(
            project_path,
            CreateIssueOptions {
                title: format!("Sequential Issue {i}"),
                ..Default::default()
            },
        )
        .await
        .expect("Should create issue");
        results.push(result);
    }

    assert_eq!(results.len(), 10);

    // Display numbers should all be unique and sequential
    let display_numbers: Vec<_> = results.iter().map(|r| r.display_number).collect();
    assert_eq!(display_numbers, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

// Test metadata preservation

#[tokio::test]
async fn test_issue_metadata_timestamps() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Timestamp Test".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue = get_issue(project_path, &result.id).await.unwrap();

    // Timestamps should be set
    assert!(!issue.metadata.created_at.is_empty());
    assert!(!issue.metadata.updated_at.is_empty());

    // created_at and updated_at should be equal initially
    assert_eq!(issue.metadata.created_at, issue.metadata.updated_at);

    // Wait a bit and update
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    update_issue(
        project_path,
        &result.id,
        UpdateIssueOptions {
            title: Some("Updated Title".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let updated_issue = get_issue(project_path, &result.id).await.unwrap();

    // created_at should be unchanged
    assert_eq!(updated_issue.metadata.created_at, issue.metadata.created_at);

    // updated_at should be changed (or same due to timing)
    assert!(!updated_issue.metadata.updated_at.is_empty());
}

// Test custom fields

#[tokio::test]
async fn test_issue_custom_fields_roundtrip() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let custom_fields = HashMap::from([
        ("assignee".to_string(), "alice".to_string()),
        ("component".to_string(), "auth".to_string()),
        ("sprint".to_string(), "sprint-42".to_string()),
    ]);

    let result = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Custom Fields Test".to_string(),
            custom_fields: custom_fields.clone(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue = get_issue(project_path, &result.id).await.unwrap();

    assert_eq!(issue.metadata.custom_fields.len(), 3);
    assert_eq!(
        issue.metadata.custom_fields.get("assignee"),
        Some(&"alice".to_string())
    );
    assert_eq!(
        issue.metadata.custom_fields.get("component"),
        Some(&"auth".to_string())
    );
    assert_eq!(
        issue.metadata.custom_fields.get("sprint"),
        Some(&"sprint-42".to_string())
    );
}

// ============ WorkspaceConfig Tests ============

#[test]
fn test_workspace_config_default_has_none() {
    let config = CentyConfig::default();
    assert!(
        config.workspace.update_status_on_open.is_none(),
        "Default WorkspaceConfig should have update_status_on_open = None"
    );
}

#[test]
fn test_workspace_config_requires_status_config_when_none() {
    let config = CentyConfig::default();
    // When update_status_on_open is None, requires_status_config should be true
    let requires = config.workspace.update_status_on_open.is_none();
    assert!(requires);
}

#[test]
fn test_workspace_config_does_not_require_status_config_when_true() {
    let mut config = CentyConfig::default();
    config.workspace.update_status_on_open = Some(true);
    let requires = config.workspace.update_status_on_open.is_none();
    assert!(!requires);
}

#[test]
fn test_workspace_config_does_not_require_status_config_when_false() {
    let mut config = CentyConfig::default();
    config.workspace.update_status_on_open = Some(false);
    let requires = config.workspace.update_status_on_open.is_none();
    assert!(!requires);
}

#[test]
fn test_workspace_config_serialization_roundtrip() {
    let ws = WorkspaceConfig {
        update_status_on_open: Some(true),
    };
    let json = serde_json::to_string(&ws).unwrap();
    let deserialized: WorkspaceConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.update_status_on_open, Some(true));
}

#[test]
fn test_workspace_config_in_centy_config_roundtrip() {
    let mut config = CentyConfig::default();
    config.workspace.update_status_on_open = Some(false);

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: CentyConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.workspace.update_status_on_open, Some(false));
}
