//! Integration tests for `GetItem` org repo fallback (issue #387).
//!
//! These tests verify that `GetItem` falls back to the org repo when an item
//! is not found in the project's own `.centy/` directory.
#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::items_after_statements,
    clippy::default_trait_access,
    clippy::let_underscore_must_use
)]

mod common;

use centy_daemon::registry::{create_organization, set_project_organization};
use centy_daemon::server::handlers::item_create::create_item;
use centy_daemon::server::handlers::item_read::get_item;
use centy_daemon::CentyConfig;
use common::create_test_dir;
use mdstore::{CreateOptions, TypeConfig};
use std::collections::HashMap;
use tokio::fs;

/// Initialize a minimal project (just the `.centy` dir and manifest).
async fn init_project(project_path: &std::path::Path) {
    let centy_path = project_path.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = centy_daemon::manifest::create_manifest();
    centy_daemon::manifest::write_manifest(project_path, &manifest)
        .await
        .unwrap();
}

/// Register project + org repo in the registry under the same org slug.
///
/// Uses the properly-locked public API to avoid races when tests run concurrently.
/// `set_project_organization` for the org repo path (which ends in `/.centy`)
/// will create a harmless nested `.centy/.centy/` org-file dir; only the
/// registry entry matters for `find_org_repo`.
async fn register_with_org(project_path: &str, org_repo_path: &str, org_slug: &str) {
    // AlreadyExists is fine when parallel tests share an org slug.
    let _ = create_organization(Some(org_slug), &format!("Org {org_slug}"), None).await;
    set_project_organization(project_path, Some(org_slug))
        .await
        .expect("assign project to org");
    set_project_organization(org_repo_path, Some(org_slug))
        .await
        .expect("assign org repo to org");
}

/// Create an item directly in a storage dir (bypassing the project `.centy` wrapper).
async fn create_item_in_dir(type_dir: &std::path::Path, title: &str) -> String {
    let config = TypeConfig::from(&centy_daemon::default_issue_config(&CentyConfig::default()));
    let item = mdstore::create(
        type_dir,
        &config,
        CreateOptions {
            title: title.to_string(),
            body: "Org body".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .expect("create item in org repo");
    item.id
}

// ─── Found in project (source is empty/project) ─────────────────────────────

#[tokio::test]
async fn test_get_item_found_in_project_source_is_empty() {
    let temp = create_test_dir();
    let project_path = temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let create_resp = create_item(centy_daemon::server::proto::CreateItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        title: "Project Issue".to_string(),
        body: String::new(),
        status: "open".to_string(),
        priority: 0,
        tags: vec![],
        custom_fields: HashMap::new(),
        projects: vec![],
        org_wide: false,
    })
    .await
    .unwrap()
    .into_inner();
    assert!(create_resp.success, "create failed: {}", create_resp.error);
    let item_id = create_resp.item.unwrap().id;

    let get_resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        display_number: None,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(get_resp.success, "get failed: {}", get_resp.error);
    assert_eq!(get_resp.item.unwrap().id, item_id);
    // source is empty (project-local, no org fallback needed)
    assert!(
        get_resp.source.is_empty() || get_resp.source == "project",
        "unexpected source: {}",
        get_resp.source
    );
}

// ─── Found in org repo (source is "org") ────────────────────────────────────

#[tokio::test]
async fn test_get_item_falls_back_to_org_repo() {
    let project_temp = create_test_dir();
    let project_path = project_temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    // Org repo: a dir whose path ends with `/.centy`
    let org_base = create_test_dir();
    let org_repo_path = org_base.path().join(".centy");
    fs::create_dir_all(&org_repo_path).await.unwrap();
    let org_pp = org_repo_path.to_str().unwrap();

    let org_slug = format!("org-387-fallback-{}", std::process::id());
    register_with_org(pp, org_pp, &org_slug).await;

    // Create an item directly in the org repo's issues dir
    let issues_dir = org_repo_path.join("issues");
    let item_id = create_item_in_dir(&issues_dir, "Org Wide Issue").await;

    // `GetItem` via the regular project — should fall back to org repo
    let get_resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        display_number: None,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(get_resp.success, "get failed: {}", get_resp.error);
    let fetched = get_resp.item.unwrap();
    assert_eq!(fetched.id, item_id);
    assert_eq!(fetched.title, "Org Wide Issue");
    assert_eq!(fetched.source, "org", "item should carry source=org");
    assert_eq!(get_resp.source, "org", "response should carry source=org");
}

// ─── Not found in either project or org repo ────────────────────────────────

#[tokio::test]
async fn test_get_item_not_found_in_either() {
    let project_temp = create_test_dir();
    let project_path = project_temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let org_base = create_test_dir();
    let org_repo_path = org_base.path().join(".centy");
    fs::create_dir_all(&org_repo_path).await.unwrap();
    let org_pp = org_repo_path.to_str().unwrap();

    let org_slug = format!("org-387-notfound-{}", std::process::id());
    register_with_org(pp, org_pp, &org_slug).await;

    let get_resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: "nonexistent-uuid-387".to_string(),
        display_number: None,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(!get_resp.success, "expected not-found error");
    assert!(get_resp.item.is_none());
}

// ─── No org repo tracked — behavior unchanged ────────────────────────────────

#[tokio::test]
async fn test_get_item_no_org_repo_unchanged_behavior() {
    let temp = create_test_dir();
    let project_path = temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    // Project has no org assignment — `find_org_repo` returns `None`
    let get_resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: "any-id-387".to_string(),
        display_number: None,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(!get_resp.success, "expected not-found error when no org repo");
    assert!(get_resp.item.is_none());
    assert!(get_resp.source.is_empty());
}

// ─── Project-local item takes precedence over org repo ──────────────────────

#[tokio::test]
async fn test_get_item_project_takes_precedence_over_org_repo() {
    let project_temp = create_test_dir();
    let project_path = project_temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let org_base = create_test_dir();
    let org_repo_path = org_base.path().join(".centy");
    fs::create_dir_all(&org_repo_path).await.unwrap();
    let org_pp = org_repo_path.to_str().unwrap();

    let org_slug = format!("org-387-precedence-{}", std::process::id());
    register_with_org(pp, org_pp, &org_slug).await;

    // Create an item in the org repo
    let org_issues_dir = org_repo_path.join("issues");
    let shared_id = create_item_in_dir(&org_issues_dir, "Org Title").await;

    // Create the same ID in the project — project item should take precedence
    let project_issues_dir = project_path.join(".centy").join("issues");
    let config = TypeConfig::from(&centy_daemon::default_issue_config(&CentyConfig::default()));
    mdstore::create(
        &project_issues_dir,
        &config,
        CreateOptions {
            title: "Project Title".to_string(),
            body: String::new(),
            id: Some(shared_id.clone()),
            status: Some("open".to_string()),
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .expect("create project item with same id");

    let get_resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: shared_id.clone(),
        display_number: None,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(get_resp.success, "get failed: {}", get_resp.error);
    let fetched = get_resp.item.unwrap();
    assert_eq!(fetched.id, shared_id);
    assert_eq!(
        fetched.title, "Project Title",
        "project item should take precedence"
    );
    assert!(
        get_resp.source.is_empty() || get_resp.source == "project",
        "source should indicate project-local, got: {}",
        get_resp.source
    );
}
