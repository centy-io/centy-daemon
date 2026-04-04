//! Integration tests for `UpdateItem` / `DeleteItem` org repo routing (issue #390).
//!
//! Verifies that both handlers transparently route writes to the org repo when
//! the target item lives there, and that project-local items are unaffected.
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
use centy_daemon::server::handlers::item_delete::delete_item;
use centy_daemon::server::handlers::item_read::get_item;
use centy_daemon::server::handlers::item_update::update_item;
use centy_daemon::server::proto::{DeleteItemRequest, GetItemRequest, UpdateItemRequest};
use centy_daemon::CentyConfig;
use common::create_test_dir;
use mdstore::{CreateOptions, TypeConfig};
use std::collections::HashMap;
use tokio::fs;

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn init_project(project_path: &std::path::Path) {
    let centy_path = project_path.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = centy_daemon::manifest::create_manifest();
    centy_daemon::manifest::write_manifest(project_path, &manifest)
        .await
        .unwrap();
}

async fn register_with_org(project_path: &str, org_repo_path: &str, org_slug: &str) {
    let _ = create_organization(Some(org_slug), &format!("Org {org_slug}"), None).await;
    set_project_organization(project_path, Some(org_slug))
        .await
        .expect("assign project to org");
    set_project_organization(org_repo_path, Some(org_slug))
        .await
        .expect("assign org repo to org");
}

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

// ─── UpdateItem ───────────────────────────────────────────────────────────────

// Update a project-local item — routing unchanged, item stays in project.
#[tokio::test]
async fn test_update_project_item_unchanged() {
    let temp = create_test_dir();
    let project_path = temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    // Create via the normal project path.
    let issues_dir = project_path.join(".centy").join("issues");
    let item_id = create_item_in_dir(&issues_dir, "Original Title").await;

    let resp = update_item(UpdateItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        title: "Updated Title".to_string(),
        body: String::new(),
        status: String::new(),
        priority: 0,
        tags: vec![],
        clear_tags: false,
        custom_fields: HashMap::new(),
        projects: vec![],
        clear_projects: false,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "update failed: {}", resp.error);
    assert_eq!(resp.item.unwrap().title, "Updated Title");

    // Verify the file is still in the project, not the org repo.
    let file_path = issues_dir.join(format!("{item_id}.md"));
    assert!(file_path.exists(), "item should remain in project");
}

// Update an org-wide item — write routed to org repo.
#[tokio::test]
async fn test_update_org_item_routes_to_org_repo() {
    let project_temp = create_test_dir();
    let project_path = project_temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let org_base = create_test_dir();
    let org_repo_path = org_base.path().join(".centy");
    fs::create_dir_all(&org_repo_path).await.unwrap();
    let org_pp = org_repo_path.to_str().unwrap();

    let org_slug = format!("org-390-update-{}", std::process::id());
    register_with_org(pp, org_pp, &org_slug).await;

    // Create item directly in the org repo.
    let issues_dir = org_repo_path.join("issues");
    let item_id = create_item_in_dir(&issues_dir, "Org Title").await;

    let resp = update_item(UpdateItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        title: "Updated Org Title".to_string(),
        body: String::new(),
        status: String::new(),
        priority: 0,
        tags: vec![],
        clear_tags: false,
        custom_fields: HashMap::new(),
        projects: vec![],
        clear_projects: false,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "update failed: {}", resp.error);
    assert_eq!(resp.item.unwrap().title, "Updated Org Title");

    // The updated file must live in the org repo, not the project.
    let org_file = issues_dir.join(format!("{item_id}.md"));
    let proj_file = project_path
        .join(".centy")
        .join("issues")
        .join(format!("{item_id}.md"));
    assert!(org_file.exists(), "item should be in org repo");
    assert!(!proj_file.exists(), "item should NOT be in project .centy");

    // Confirm the content persisted by reading back.
    let get_resp = get_item(GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        display_number: None,
    })
    .await
    .unwrap()
    .into_inner();
    assert!(
        get_resp.success,
        "get after update failed: {}",
        get_resp.error
    );
    assert_eq!(get_resp.item.unwrap().title, "Updated Org Title");
}

// UpdateItem on an item absent from both project and org repo → not-found error.
#[tokio::test]
async fn test_update_item_not_found_in_either() {
    let project_temp = create_test_dir();
    let project_path = project_temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let org_base = create_test_dir();
    let org_repo_path = org_base.path().join(".centy");
    fs::create_dir_all(&org_repo_path).await.unwrap();
    let org_pp = org_repo_path.to_str().unwrap();

    let org_slug = format!("org-390-upd-notfound-{}", std::process::id());
    register_with_org(pp, org_pp, &org_slug).await;

    let resp = update_item(UpdateItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: "nonexistent-uuid-390".to_string(),
        title: "Should Fail".to_string(),
        body: String::new(),
        status: String::new(),
        priority: 0,
        tags: vec![],
        clear_tags: false,
        custom_fields: HashMap::new(),
        projects: vec![],
        clear_projects: false,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(!resp.success, "expected not-found error");
    assert!(resp.item.is_none());
}

// UpdateItem when no org repo is tracked — unchanged behavior (not-found error).
#[tokio::test]
async fn test_update_item_no_org_repo_unchanged_behavior() {
    let temp = create_test_dir();
    let project_path = temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let resp = update_item(UpdateItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: "missing-uuid-390".to_string(),
        title: "Should Fail".to_string(),
        body: String::new(),
        status: String::new(),
        priority: 0,
        tags: vec![],
        clear_tags: false,
        custom_fields: HashMap::new(),
        projects: vec![],
        clear_projects: false,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(!resp.success, "expected not-found when no org repo");
    assert!(resp.item.is_none());
}

// ─── DeleteItem ───────────────────────────────────────────────────────────────

// Delete a project-local item — routing unchanged.
#[tokio::test]
async fn test_delete_project_item_unchanged() {
    let temp = create_test_dir();
    let project_path = temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let issues_dir = project_path.join(".centy").join("issues");
    let item_id = create_item_in_dir(&issues_dir, "To Delete").await;

    let resp = delete_item(DeleteItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        force: true,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "delete failed: {}", resp.error);

    // Hard-deleted — file must be gone.
    let file_path = issues_dir.join(format!("{item_id}.md"));
    assert!(!file_path.exists(), "file should be deleted from project");
}

// Delete an org-wide item — deletion routed to org repo.
#[tokio::test]
async fn test_delete_org_item_routes_to_org_repo() {
    let project_temp = create_test_dir();
    let project_path = project_temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let org_base = create_test_dir();
    let org_repo_path = org_base.path().join(".centy");
    fs::create_dir_all(&org_repo_path).await.unwrap();
    let org_pp = org_repo_path.to_str().unwrap();

    let org_slug = format!("org-390-delete-{}", std::process::id());
    register_with_org(pp, org_pp, &org_slug).await;

    // Create item directly in the org repo.
    let issues_dir = org_repo_path.join("issues");
    let item_id = create_item_in_dir(&issues_dir, "Org Item To Delete").await;

    let resp = delete_item(DeleteItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        force: true,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "delete failed: {}", resp.error);

    // File must be gone from org repo.
    let org_file = issues_dir.join(format!("{item_id}.md"));
    assert!(!org_file.exists(), "item should be deleted from org repo");
}

// DeleteItem on an item absent from both → not-found error.
#[tokio::test]
async fn test_delete_item_not_found_in_either() {
    let project_temp = create_test_dir();
    let project_path = project_temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let org_base = create_test_dir();
    let org_repo_path = org_base.path().join(".centy");
    fs::create_dir_all(&org_repo_path).await.unwrap();
    let org_pp = org_repo_path.to_str().unwrap();

    let org_slug = format!("org-390-del-notfound-{}", std::process::id());
    register_with_org(pp, org_pp, &org_slug).await;

    let resp = delete_item(DeleteItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: "nonexistent-uuid-390-del".to_string(),
        force: true,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(!resp.success, "expected not-found error");
}

// DeleteItem when no org repo is tracked — unchanged behavior.
#[tokio::test]
async fn test_delete_item_no_org_repo_unchanged_behavior() {
    let temp = create_test_dir();
    let project_path = temp.path();
    init_project(project_path).await;
    let pp = project_path.to_str().unwrap();

    let resp = delete_item(DeleteItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: "missing-uuid-390-del".to_string(),
        force: true,
    })
    .await
    .unwrap()
    .into_inner();

    assert!(!resp.success, "expected not-found when no org repo");
}
