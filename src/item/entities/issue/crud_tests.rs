#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

// Test setup helpers
use std::sync::LazyLock;

static CRUD_TEST_ISOLATED_HOME: LazyLock<()> = LazyLock::new(|| {
    let dir = tempfile::tempdir().expect("Failed to create isolated centy home");
    std::env::set_var("CENTY_HOME", dir.path());
    Box::leak(Box::new(dir));
});

fn ensure_test_isolation() {
    LazyLock::force(&CRUD_TEST_ISOLATED_HOME);
}

async fn init_project(project_path: &std::path::Path) {
    use crate::reconciliation::{execute_reconciliation, ReconciliationDecisions};
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(project_path, decisions, true)
        .await
        .expect("Failed to init project");
}

// --- read.rs: read_issue_from_frontmatter tests ---

#[tokio::test]
async fn test_read_issue_from_frontmatter_valid_file() {
    use crate::item::entities::issue::create::{create_issue, CreateIssueOptions};
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    let result = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Frontmatter Test".to_string(),
            description: "Body text".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let issue_file = project_path.join(format!(".centy/issues/{}.md", result.id));
    let issue = read_issue_from_frontmatter(&issue_file, &result.id)
        .await
        .expect("Should read issue");

    assert_eq!(issue.id, result.id);
    assert_eq!(issue.title, "Frontmatter Test");
    assert_eq!(issue.description, "Body text");
    assert_eq!(issue.metadata.display_number, 1);
}

#[tokio::test]
async fn test_read_issue_from_legacy_folder_missing_files() {
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_folder = temp_dir.path().join("fake-issue");
    tokio::fs::create_dir_all(&fake_folder).await.unwrap();

    let result = read_issue_from_legacy_folder(&fake_folder, "fake-uuid").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        IssueCrudError::InvalidIssueFormat(_)
    ));
}

#[tokio::test]
async fn test_read_issue_from_legacy_folder_valid() {
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let issue_folder = temp_dir.path().join("legacy-issue");
    tokio::fs::create_dir_all(&issue_folder).await.unwrap();

    // Write a legacy issue.md and metadata.json
    let issue_md = "# Legacy Issue\n\nLegacy description.";
    tokio::fs::write(issue_folder.join("issue.md"), issue_md)
        .await
        .unwrap();
    let metadata_json = r#"{
        "displayNumber": 5,
        "status": "open",
        "priority": 2,
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T00:00:00Z"
    }"#;
    tokio::fs::write(issue_folder.join("metadata.json"), metadata_json)
        .await
        .unwrap();

    let issue = read_issue_from_legacy_folder(&issue_folder, "legacy-uuid")
        .await
        .expect("Should read legacy issue");

    assert_eq!(issue.id, "legacy-uuid");
    assert_eq!(issue.title, "Legacy Issue");
    assert_eq!(issue.description, "Legacy description.");
    assert_eq!(issue.metadata.display_number, 5);
    assert_eq!(issue.metadata.status, "open");
    assert_eq!(issue.metadata.priority, 2);
}

// --- get.rs: get_issue and get_issue_by_display_number tests ---

#[tokio::test]
async fn test_get_issue_not_initialized() {
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let result = get_issue(temp_dir.path(), "some-id").await;
    assert!(matches!(result, Err(IssueCrudError::NotInitialized)));
}

#[tokio::test]
async fn test_get_issue_by_display_number_success() {
    use crate::item::entities::issue::create::{create_issue, CreateIssueOptions};
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Display Number Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let issue = get_issue_by_display_number(project_path, created.display_number)
        .await
        .expect("Should get by display number");

    assert_eq!(issue.id, created.id);
    assert_eq!(issue.metadata.display_number, 1);
}

#[tokio::test]
async fn test_get_issue_by_display_number_not_found() {
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    let result = get_issue_by_display_number(project_path, 999).await;
    assert!(matches!(
        result,
        Err(IssueCrudError::IssueDisplayNumberNotFound(999))
    ));
}

#[tokio::test]
async fn test_get_issue_by_display_number_no_issues_dir() {
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    // Remove issues directory to test that code path
    let issues_path = project_path.join(".centy/issues");
    tokio::fs::remove_dir_all(&issues_path).await.unwrap();

    let result = get_issue_by_display_number(project_path, 1).await;
    assert!(matches!(
        result,
        Err(IssueCrudError::IssueDisplayNumberNotFound(1))
    ));
}

// --- list.rs: list_issues and get_issues_by_uuid tests ---

#[tokio::test]
async fn test_list_issues_not_initialized() {
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let result = list_issues(temp_dir.path(), None, None, None, false).await;
    assert!(matches!(result, Err(IssueCrudError::NotInitialized)));
}

#[tokio::test]
async fn test_list_issues_with_filters() {
    use crate::item::entities::issue::create::{create_issue, CreateIssueOptions};
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    // Create a draft issue
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Draft Issue".to_string(),
            draft: Some(true),
            ..Default::default()
        },
    )
    .await
    .expect("Should create draft issue");

    // Create a non-draft issue
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Normal Issue".to_string(),
            draft: Some(false),
            ..Default::default()
        },
    )
    .await
    .expect("Should create normal issue");

    // Filter by draft=true
    let drafts = list_issues(project_path, None, None, Some(true), false)
        .await
        .expect("Should list drafts");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].title, "Draft Issue");

    // Filter by draft=false
    let non_drafts = list_issues(project_path, None, None, Some(false), false)
        .await
        .expect("Should list non-drafts");
    assert_eq!(non_drafts.len(), 1);
    assert_eq!(non_drafts[0].title, "Normal Issue");
}

#[tokio::test]
async fn test_get_issues_by_uuid_invalid_format() {
    let result = get_issues_by_uuid("not-a-uuid", &[]).await;
    assert!(matches!(result, Err(IssueCrudError::InvalidIssueFormat(_))));
}

#[tokio::test]
async fn test_get_issues_by_uuid_skips_uninitialized_projects() {
    use crate::registry::ProjectInfo;
    ensure_test_isolation();

    // Use a valid UUID format
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    let projects = vec![ProjectInfo {
        path: "/nonexistent/path".to_string(),
        name: Some("test-project".to_string()),
        initialized: false,
        organization_slug: None,
        first_accessed: "2024-01-01".to_string(),
        last_accessed: "2024-01-01".to_string(),
        issue_count: 0,
        doc_count: 0,
        is_favorite: false,
        is_archived: false,
        organization_name: None,
        user_title: None,
        project_title: None,
        project_version: None,
        project_behind: false,
    }];
    let result = get_issues_by_uuid(uuid, &projects)
        .await
        .expect("Should succeed");
    assert!(result.issues.is_empty());
    assert!(result.errors.is_empty());
}

#[tokio::test]
async fn test_get_issues_by_uuid_finds_issue() {
    use crate::item::entities::issue::create::{create_issue, CreateIssueOptions};
    use crate::registry::ProjectInfo;
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "UUID Search Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let projects = vec![ProjectInfo {
        path: project_path.to_string_lossy().to_string(),
        name: Some("test-project".to_string()),
        initialized: true,
        organization_slug: None,
        first_accessed: "2024-01-01".to_string(),
        last_accessed: "2024-01-01".to_string(),
        issue_count: 0,
        doc_count: 0,
        is_favorite: false,
        is_archived: false,
        organization_name: None,
        user_title: None,
        project_title: None,
        project_version: None,
        project_behind: false,
    }];

    let result = get_issues_by_uuid(&created.id, &projects)
        .await
        .expect("Should succeed");
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0].issue.id, created.id);
    assert_eq!(result.issues[0].project_name, "test-project");
}

#[tokio::test]
async fn test_get_issues_by_uuid_project_name_from_path_when_none() {
    use crate::item::entities::issue::create::{create_issue, CreateIssueOptions};
    use crate::registry::ProjectInfo;
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "No-Name Project Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let projects = vec![ProjectInfo {
        path: project_path.to_string_lossy().to_string(),
        name: None, // No name - should fall back to path's last component
        initialized: true,
        organization_slug: None,
        first_accessed: "2024-01-01".to_string(),
        last_accessed: "2024-01-01".to_string(),
        issue_count: 0,
        doc_count: 0,
        is_favorite: false,
        is_archived: false,
        organization_name: None,
        user_title: None,
        project_title: None,
        project_version: None,
        project_behind: false,
    }];

    let result = get_issues_by_uuid(&created.id, &projects)
        .await
        .expect("Should succeed");
    assert_eq!(result.issues.len(), 1);
    // project_name should be derived from path since name is None
    assert!(!result.issues[0].project_name.is_empty());
}

// --- update_helpers.rs tests ---

#[tokio::test]
async fn test_resolve_update_options_preserves_defaults() {
    use crate::item::entities::issue::create::{create_issue, CreateIssueOptions};
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Original".to_string(),
            description: "Description".to_string(),
            priority: Some(1),
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let issue = get_issue(project_path, &created.id)
        .await
        .expect("Should get issue");

    let applied = resolve_update_options(&issue, UpdateIssueOptions::default(), project_path, 3)
        .await
        .expect("Should resolve");

    assert_eq!(applied.title, "Original");
    assert_eq!(applied.description, "Description");
    assert_eq!(applied.status, "open");
    assert_eq!(applied.priority, 1);
}

#[tokio::test]
async fn test_resolve_update_options_with_new_priority() {
    use crate::item::entities::issue::create::{create_issue, CreateIssueOptions};
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Priority Test".to_string(),
            priority: Some(2),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let issue = get_issue(project_path, &created.id)
        .await
        .expect("Should get issue");

    let applied = resolve_update_options(
        &issue,
        UpdateIssueOptions {
            priority: Some(3),
            ..Default::default()
        },
        project_path,
        3,
    )
    .await
    .expect("Should resolve");

    assert_eq!(applied.priority, 3);
}

#[tokio::test]
async fn test_resolve_update_options_custom_fields_merge() {
    use crate::item::entities::issue::create::{create_issue, CreateIssueOptions};
    ensure_test_isolation();

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let project_path = temp_dir.path();
    init_project(project_path).await;

    let mut initial_fields = std::collections::HashMap::new();
    initial_fields.insert("existing".to_string(), "value".to_string());

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Custom Fields Test".to_string(),
            custom_fields: initial_fields,
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let issue = get_issue(project_path, &created.id)
        .await
        .expect("Should get issue");

    let mut new_fields = std::collections::HashMap::new();
    new_fields.insert("new-field".to_string(), "new-value".to_string());

    let applied = resolve_update_options(
        &issue,
        UpdateIssueOptions {
            custom_fields: new_fields,
            ..Default::default()
        },
        project_path,
        3,
    )
    .await
    .expect("Should resolve");

    assert_eq!(
        applied.custom_fields.get("new-field").map(String::as_str),
        Some("new-value")
    );
}


#[test]
fn test_parse_issue_md_with_description() {
    let content = "# My Issue Title\n\nThis is the description.\nWith multiple lines.";
    let (title, description) = parse_issue_md(content);
    assert_eq!(title, "My Issue Title");
    assert_eq!(
        description,
        "This is the description.\nWith multiple lines."
    );
}

#[test]
fn test_parse_issue_md_title_only() {
    let content = "# My Issue Title\n";
    let (title, description) = parse_issue_md(content);
    assert_eq!(title, "My Issue Title");
    assert_eq!(description, "");
}

#[test]
fn test_parse_issue_md_empty() {
    let content = "";
    let (title, description) = parse_issue_md(content);
    assert_eq!(title, "");
    assert_eq!(description, "");
}
