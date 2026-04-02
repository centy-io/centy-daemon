//! Tests for `org_issues`: paths, config, crud, `crud_list`, `crud_ops`.
//!
//! All tests use `acquire_org_test_lock()` from the parent `organizations`
//! module so that every test in the entire `registry::organizations` tree is
//! serialized and shares a single stable `CENTY_HOME`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]

/// Acquire the shared org-module test lock.
fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    super::super::acquire_org_test_lock()
}

// ─── paths tests ────────────────────────────────────────────────────────────

#[test]
fn test_get_org_dir_uses_centy_home() {
    let _lock = acquire_lock();
    let result = super::paths::get_org_dir("my-org").expect("should succeed");
    assert!(
        result.ends_with("orgs/my-org"),
        "path should end with orgs/my-org, got: {result:?}"
    );
}

#[test]
fn test_get_org_dir_falls_back_to_home() {
    // This test temporarily removes CENTY_HOME to exercise the HOME fallback.
    // We hold the lock so no other test can race on CENTY_HOME.
    let _lock = acquire_lock();

    let saved = std::env::var("CENTY_HOME").ok();
    std::env::remove_var("CENTY_HOME");

    let result = super::paths::get_org_dir("acme");
    if std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok() {
        let path = result.expect("should succeed with HOME set");
        let s = path.to_string_lossy();
        assert!(s.contains(".centy"), "should contain .centy: {s}");
        assert!(s.ends_with("acme"), "should end with slug: {s}");
    } else {
        assert!(result.is_err(), "should error when no HOME");
    }

    // Restore CENTY_HOME so subsequent tests remain isolated.
    if let Some(v) = saved {
        std::env::set_var("CENTY_HOME", v);
    }
}

#[test]
fn test_get_org_issues_dir() {
    let _lock = acquire_lock();
    let result = super::paths::get_org_issues_dir("my-org").expect("should succeed");
    assert!(result.ends_with("orgs/my-org/issues"), "got: {result:?}");
}

#[test]
fn test_get_org_config_path() {
    let _lock = acquire_lock();
    let result = super::paths::get_org_config_path("my-org").expect("should succeed");
    assert!(
        result.ends_with("orgs/my-org/config.json"),
        "got: {result:?}"
    );
}

// ─── config tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_org_config_returns_default_when_missing() {
    let _lock = acquire_lock();

    let cfg = super::config::get_org_config("cfg-no-such-org")
        .await
        .expect("should succeed with defaults");
    assert_eq!(cfg.priority_levels, 3);
    assert!(cfg.custom_fields.is_empty());
}

#[tokio::test]
async fn test_update_and_get_org_config() {
    let _lock = acquire_lock();

    let cfg = super::config::OrgConfig {
        priority_levels: 5,
        custom_fields: vec![super::config::OrgCustomFieldDef {
            name: "team".to_string(),
            default_value: Some("backend".to_string()),
            description: Some("Engineering team".to_string()),
        }],
    };

    super::config::update_org_config("cfg-acme", &cfg)
        .await
        .expect("should write config");

    let loaded = super::config::get_org_config("cfg-acme")
        .await
        .expect("should read config");
    assert_eq!(loaded.priority_levels, 5);
    assert_eq!(loaded.custom_fields.len(), 1);
    assert_eq!(loaded.custom_fields[0].name, "team");
    assert_eq!(
        loaded.custom_fields[0].default_value,
        Some("backend".to_string())
    );
    assert_eq!(
        loaded.custom_fields[0].description,
        Some("Engineering team".to_string())
    );
}

#[tokio::test]
async fn test_update_org_config_overwrites() {
    let _lock = acquire_lock();

    let cfg1 = super::config::OrgConfig {
        priority_levels: 2,
        custom_fields: vec![],
    };
    super::config::update_org_config("cfg-acme2", &cfg1)
        .await
        .expect("write 1");

    let cfg2 = super::config::OrgConfig {
        priority_levels: 7,
        custom_fields: vec![],
    };
    super::config::update_org_config("cfg-acme2", &cfg2)
        .await
        .expect("write 2");

    let loaded = super::config::get_org_config("cfg-acme2")
        .await
        .expect("read");
    assert_eq!(loaded.priority_levels, 7);
}

#[tokio::test]
async fn test_get_org_config_invalid_json_returns_error() {
    let _lock = acquire_lock();

    // Get the current CENTY_HOME and write bad JSON there.
    let centy_home = std::env::var("CENTY_HOME").expect("CENTY_HOME set");
    let config_path = std::path::Path::new(&centy_home)
        .join("orgs")
        .join("cfg-bad-org")
        .join("config.json");
    tokio::fs::create_dir_all(config_path.parent().unwrap())
        .await
        .expect("create dirs");
    tokio::fs::write(&config_path, b"not valid json { }")
        .await
        .expect("write bad json");

    let result = super::config::get_org_config("cfg-bad-org").await;
    assert!(result.is_err(), "should error on invalid JSON");

    // Clean up so other tests don't see this bad file.
    drop(tokio::fs::remove_file(&config_path).await);
}

// ─── crud_ops tests (create / get) ──────────────────────────────────────────

#[tokio::test]
async fn test_create_org_issue_basic() {
    let _lock = acquire_lock();

    let issue = super::crud_ops::create_org_issue(
        "ops-test-org",
        "My first issue",
        "Some description",
        1,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("create should succeed");

    assert_eq!(issue.title, "My first issue");
    assert_eq!(issue.description, "Some description");
    assert_eq!(issue.status, "open");
    assert_eq!(issue.priority, 1);
    assert!(!issue.id.is_empty());
    assert!(
        issue.display_number >= 1,
        "display_number should be positive"
    );
    assert!(issue.referenced_projects.is_empty());
}

#[tokio::test]
async fn test_create_org_issue_empty_title_fails() {
    let _lock = acquire_lock();

    let result = super::crud_ops::create_org_issue(
        "ops-test-org2",
        "   ",
        "desc",
        1,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await;

    assert!(result.is_err(), "empty title should fail");
}

#[tokio::test]
async fn test_create_org_issue_with_custom_fields_and_projects() {
    let _lock = acquire_lock();

    let mut fields = std::collections::HashMap::new();
    fields.insert("team".to_string(), "backend".to_string());

    let issue = super::crud_ops::create_org_issue(
        "ops-test-org3",
        "Issue with extras",
        "desc",
        2,
        "in-progress",
        fields,
        vec!["proj-a".to_string(), "proj-b".to_string()],
    )
    .await
    .expect("should create");

    assert_eq!(
        issue.custom_fields.get("team").map(String::as_str),
        Some("backend")
    );
    assert_eq!(issue.referenced_projects, vec!["proj-a", "proj-b"]);
}

#[tokio::test]
async fn test_get_org_issue_not_found() {
    let _lock = acquire_lock();

    let result = super::crud_ops::get_org_issue("ops-test-org4", "nonexistent-id").await;
    assert!(result.is_err(), "should error for missing issue");
}

#[tokio::test]
async fn test_get_org_issue_roundtrip() {
    let _lock = acquire_lock();

    let created = super::crud_ops::create_org_issue(
        "ops-test-org5",
        "Round trip",
        "body text",
        3,
        "closed",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("create");

    let fetched = super::crud_ops::get_org_issue("ops-test-org5", &created.id)
        .await
        .expect("get");

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "Round trip");
    assert_eq!(fetched.description, "body text");
    assert_eq!(fetched.status, "closed");
    assert_eq!(fetched.priority, 3);
}

// ─── crud tests (update / delete) ───────────────────────────────────────────

async fn create_test_issue(org: &str) -> String {
    let issue = super::crud_ops::create_org_issue(
        org,
        "Test title",
        "desc",
        1,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("create");
    issue.id
}

#[tokio::test]
async fn test_update_title() {
    let _lock = acquire_lock();
    let id = create_test_issue("crud-upd-org").await;
    let updated = super::crud::update_org_issue(
        "crud-upd-org",
        &id,
        super::crud::UpdateOrgIssueOptions {
            title: Some("New title".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("update");
    assert_eq!(updated.title, "New title");
}

#[tokio::test]
async fn test_update_description() {
    let _lock = acquire_lock();
    let id = create_test_issue("crud-upd-org2").await;
    let updated = super::crud::update_org_issue(
        "crud-upd-org2",
        &id,
        super::crud::UpdateOrgIssueOptions {
            description: Some("New desc".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("update");
    assert_eq!(updated.description, "New desc");
}

#[tokio::test]
async fn test_update_status() {
    let _lock = acquire_lock();
    let id = create_test_issue("crud-upd-org3").await;
    let updated = super::crud::update_org_issue(
        "crud-upd-org3",
        &id,
        super::crud::UpdateOrgIssueOptions {
            status: Some("closed".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("update");
    assert_eq!(updated.status, "closed");
}

#[tokio::test]
async fn test_update_priority() {
    let _lock = acquire_lock();
    let id = create_test_issue("crud-upd-org4").await;
    let updated = super::crud::update_org_issue(
        "crud-upd-org4",
        &id,
        super::crud::UpdateOrgIssueOptions {
            priority: Some(5),
            ..Default::default()
        },
    )
    .await
    .expect("update");
    assert_eq!(updated.priority, 5);
}

#[tokio::test]
async fn test_update_custom_fields() {
    let _lock = acquire_lock();
    let id = create_test_issue("crud-upd-org5").await;

    let mut fields = std::collections::HashMap::new();
    fields.insert("key".to_string(), "value".to_string());

    let updated = super::crud::update_org_issue(
        "crud-upd-org5",
        &id,
        super::crud::UpdateOrgIssueOptions {
            custom_fields: Some(fields),
            ..Default::default()
        },
    )
    .await
    .expect("update");

    assert_eq!(
        updated.custom_fields.get("key").map(String::as_str),
        Some("value")
    );
}

#[tokio::test]
async fn test_add_and_remove_referenced_projects() {
    let _lock = acquire_lock();
    let id = create_test_issue("crud-upd-org6").await;

    // Add projects
    let updated = super::crud::update_org_issue(
        "crud-upd-org6",
        &id,
        super::crud::UpdateOrgIssueOptions {
            add_referenced_projects: vec!["proj-a".to_string(), "proj-b".to_string()],
            ..Default::default()
        },
    )
    .await
    .expect("add projects");

    assert!(updated.referenced_projects.contains(&"proj-a".to_string()));
    assert!(updated.referenced_projects.contains(&"proj-b".to_string()));

    // Adding same project again is idempotent
    let updated2 = super::crud::update_org_issue(
        "crud-upd-org6",
        &id,
        super::crud::UpdateOrgIssueOptions {
            add_referenced_projects: vec!["proj-a".to_string()],
            ..Default::default()
        },
    )
    .await
    .expect("add again");
    assert_eq!(
        updated2
            .referenced_projects
            .iter()
            .filter(|p| p.as_str() == "proj-a")
            .count(),
        1,
        "should not duplicate"
    );

    // Remove a project
    let updated3 = super::crud::update_org_issue(
        "crud-upd-org6",
        &id,
        super::crud::UpdateOrgIssueOptions {
            remove_referenced_projects: vec!["proj-a".to_string()],
            ..Default::default()
        },
    )
    .await
    .expect("remove project");

    assert!(!updated3.referenced_projects.contains(&"proj-a".to_string()));
    assert!(updated3.referenced_projects.contains(&"proj-b".to_string()));
}

#[tokio::test]
async fn test_update_org_issue_not_found() {
    let _lock = acquire_lock();
    let result = super::crud::update_org_issue(
        "crud-upd-org7",
        "does-not-exist",
        super::crud::UpdateOrgIssueOptions::default(),
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_org_issue() {
    let _lock = acquire_lock();
    let id = create_test_issue("crud-del-org").await;
    super::crud::delete_org_issue("crud-del-org", &id)
        .await
        .expect("delete should succeed");
    let result = super::crud_ops::get_org_issue("crud-del-org", &id).await;
    assert!(result.is_err(), "should be gone");
}

#[tokio::test]
async fn test_delete_org_issue_not_found() {
    let _lock = acquire_lock();
    let result = super::crud::delete_org_issue("crud-del-org2", "nonexistent").await;
    assert!(result.is_err());
}

// ─── crud_list tests ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_org_issues_empty_when_no_dir() {
    let _lock = acquire_lock();
    let issues = super::crud_list::list_org_issues(
        "list-empty-org",
        super::crud_types::ListOrgIssuesOptions::default(),
    )
    .await
    .expect("should return empty vec");
    assert!(issues.is_empty());
}

#[tokio::test]
async fn test_list_org_issues_returns_all() {
    let _lock = acquire_lock();
    let org = "list-all-org";
    super::crud_ops::create_org_issue(
        org,
        "Issue 1",
        "d1",
        1,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("c1");
    super::crud_ops::create_org_issue(
        org,
        "Issue 2",
        "d2",
        2,
        "closed",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("c2");

    let issues =
        super::crud_list::list_org_issues(org, super::crud_types::ListOrgIssuesOptions::default())
            .await
            .expect("list");
    assert!(issues.len() >= 2);
    // Should be sorted by display_number
    for w in issues.windows(2) {
        if let [a, b] = w {
            assert!(a.display_number <= b.display_number);
        }
    }
}

#[tokio::test]
async fn test_list_org_issues_filter_by_status() {
    let _lock = acquire_lock();
    let org = "list-status-org";
    super::crud_ops::create_org_issue(
        org,
        "Open issue",
        "d",
        1,
        "status-open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("c1");
    super::crud_ops::create_org_issue(
        org,
        "Closed issue",
        "d",
        1,
        "status-closed",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("c2");

    let open = super::crud_list::list_org_issues(
        org,
        super::crud_types::ListOrgIssuesOptions {
            status: Some("status-open".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("filter open");
    assert!(open.iter().all(|i| i.status == "status-open"));
    assert!(!open.is_empty());

    let closed = super::crud_list::list_org_issues(
        org,
        super::crud_types::ListOrgIssuesOptions {
            status: Some("status-closed".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("filter closed");
    assert!(closed.iter().all(|i| i.status == "status-closed"));
    assert!(!closed.is_empty());
}

#[tokio::test]
async fn test_list_org_issues_filter_by_priority() {
    let _lock = acquire_lock();
    let org = "list-prio-org";
    super::crud_ops::create_org_issue(
        org,
        "Prio 99 issue",
        "d",
        99,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("c1");
    super::crud_ops::create_org_issue(
        org,
        "Prio 88 issue",
        "d",
        88,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("c2");

    let p99 = super::crud_list::list_org_issues(
        org,
        super::crud_types::ListOrgIssuesOptions {
            priority: Some(99),
            ..Default::default()
        },
    )
    .await
    .expect("filter p99");
    assert!(!p99.is_empty());
    assert!(p99.iter().all(|i| i.priority == 99));
}

#[tokio::test]
async fn test_list_org_issues_filter_by_referenced_project() {
    let _lock = acquire_lock();
    let org = "list-proj-org";
    super::crud_ops::create_org_issue(
        org,
        "Issue with proj",
        "d",
        1,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec!["alpha-proj".to_string()],
    )
    .await
    .expect("c1");
    super::crud_ops::create_org_issue(
        org,
        "Issue without proj",
        "d",
        1,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("c2");

    let filtered = super::crud_list::list_org_issues(
        org,
        super::crud_types::ListOrgIssuesOptions {
            referenced_project: Some("alpha-proj".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("filter proj");
    assert!(!filtered.is_empty());
    assert!(filtered
        .iter()
        .all(|i| i.referenced_projects.contains(&"alpha-proj".to_string())));
}

#[tokio::test]
async fn test_get_org_issue_by_display_number_found() {
    let _lock = acquire_lock();
    let org = "dn-lookup-org";
    let created = super::crud_ops::create_org_issue(
        org,
        "DN issue",
        "d",
        1,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("create");

    let found = super::crud_list::get_org_issue_by_display_number(org, created.display_number)
        .await
        .expect("should find");
    assert_eq!(found.id, created.id);
    assert_eq!(found.title, "DN issue");
}

#[tokio::test]
async fn test_get_org_issue_by_display_number_not_found_empty_dir() {
    let _lock = acquire_lock();
    let result = super::crud_list::get_org_issue_by_display_number("dn-no-issues-org", 99999).await;
    assert!(result.is_err());
}

// ─── error mapping tests for variants requiring private path types ────────────

#[test]
fn test_org_issue_error_path_error_code() {
    use crate::registry::OrgIssueError;
    use crate::server::error_mapping::ToStructuredError as _;
    let err = OrgIssueError::PathError(super::paths::PathError::HomeDirNotFound);
    let (code, _) = err.error_code_and_tip();
    assert_eq!(code, "PATH_ERROR");
}

#[test]
fn test_org_issue_error_org_registry_error_code() {
    use crate::item::entities::issue::org_registry::OrgIssueRegistryError;
    use crate::registry::OrgIssueError;
    use crate::server::error_mapping::ToStructuredError as _;
    let err = OrgIssueError::OrgRegistryError(OrgIssueRegistryError::HomeDirNotFound);
    let (code, _) = err.error_code_and_tip();
    assert_eq!(code, "ORG_REGISTRY_ERROR");
}

#[test]
fn test_org_config_error_path_error_code() {
    use crate::registry::OrgConfigError;
    use crate::server::error_mapping::ToStructuredError as _;
    let err = OrgConfigError::PathError(super::paths::PathError::HomeDirNotFound);
    let (code, _) = err.error_code_and_tip();
    assert_eq!(code, "PATH_ERROR");
}

#[tokio::test]
async fn test_get_org_issue_by_display_number_not_found_wrong_number() {
    let _lock = acquire_lock();
    let org = "dn-wrong-num-org";
    super::crud_ops::create_org_issue(
        org,
        "Only issue",
        "d",
        1,
        "open",
        std::collections::HashMap::<String, String>::new(),
        vec![],
    )
    .await
    .expect("create");

    let result = super::crud_list::get_org_issue_by_display_number(org, 999_999).await;
    assert!(result.is_err());
}
