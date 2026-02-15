#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing
)]

mod common;

use centy_daemon::server::handlers::item_create::create_item;
use centy_daemon::server::handlers::item_delete::delete_item;
use centy_daemon::server::handlers::item_list::list_items;
use centy_daemon::server::handlers::item_read::get_item;
use centy_daemon::server::handlers::item_restore::restore_item;
use centy_daemon::server::handlers::item_soft_delete::soft_delete_item;
use centy_daemon::server::handlers::item_type_resolve::normalize_item_type;
use centy_daemon::server::handlers::item_update::update_item;
use common::create_test_dir;
use std::collections::HashMap;
use tokio::fs;

/// Initialize a minimal project for handler tests.
async fn init_project(project_path: &std::path::Path) {
    let centy_path = project_path.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();

    let manifest = centy_daemon::manifest::create_manifest();
    centy_daemon::manifest::write_manifest(project_path, &manifest)
        .await
        .unwrap();
}

/// Helper to build a CreateItemRequest proto message.
fn create_req(
    project_path: &str,
    item_type: &str,
    title: &str,
    body: &str,
    status: &str,
    priority: i32,
    custom_fields: HashMap<String, String>,
) -> centy_daemon::server::proto::CreateItemRequest {
    centy_daemon::server::proto::CreateItemRequest {
        project_path: project_path.to_string(),
        item_type: item_type.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        status: status.to_string(),
        priority,
        custom_fields,
    }
}

// ─── Create + Get roundtrip with "issues" type ──────────────────────────────

#[tokio::test]
async fn test_create_and_get_issue_type() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = create_item(create_req(
        pp,
        "issues",
        "Test Issue",
        "Body text",
        "open",
        2,
        HashMap::new(),
    ))
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "create failed: {}", resp.error);
    let item = resp.item.unwrap();
    assert_eq!(item.title, "Test Issue");
    assert_eq!(item.body, "Body text");
    assert_eq!(item.item_type, "issues");
    let meta = item.metadata.unwrap();
    assert_eq!(meta.display_number, 1);
    assert_eq!(meta.status, "open");
    assert_eq!(meta.priority, 2);
    assert!(!meta.created_at.is_empty());
    assert!(meta.deleted_at.is_empty());

    // Get it back
    let get_resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item.id.clone(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(get_resp.success, "get failed: {}", get_resp.error);
    let fetched = get_resp.item.unwrap();
    assert_eq!(fetched.id, item.id);
    assert_eq!(fetched.title, "Test Issue");
}

// ─── Create + Get with "docs" type (slug-based) ─────────────────────────────

#[tokio::test]
async fn test_create_and_get_doc_type() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = create_item(create_req(
        pp,
        "docs",
        "Getting Started",
        "Welcome!",
        "",
        0,
        HashMap::new(),
    ))
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "create failed: {}", resp.error);
    let item = resp.item.unwrap();
    assert_eq!(item.id, "getting-started");
    assert_eq!(item.item_type, "docs");
    let meta = item.metadata.unwrap();
    // Docs have no display_number, status, or priority
    assert_eq!(meta.display_number, 0);
    assert!(meta.status.is_empty());
    assert_eq!(meta.priority, 0);

    // Get by slug
    let get_resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "docs".to_string(),
        item_id: "getting-started".to_string(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(get_resp.success);
    assert_eq!(get_resp.item.unwrap().title, "Getting Started");
}

// ─── List with filters ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_with_filters() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    // Create multiple items
    for (title, status, priority) in [
        ("Open P1", "open", 1),
        ("Open P2", "open", 2),
        ("Closed P1", "closed", 1),
    ] {
        let resp = create_item(create_req(
            pp,
            "issues",
            title,
            "",
            status,
            priority,
            HashMap::new(),
        ))
        .await
        .unwrap()
        .into_inner();
        assert!(resp.success, "create failed: {}", resp.error);
    }

    // List all
    let resp = list_items(centy_daemon::server::proto::ListItemsRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        status: String::new(),
        priority: 0,
        include_deleted: false,
        limit: 0,
        offset: 0,
    })
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success);
    assert_eq!(resp.total_count, 3);

    // Filter by status
    let resp = list_items(centy_daemon::server::proto::ListItemsRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        status: "open".to_string(),
        priority: 0,
        include_deleted: false,
        limit: 0,
        offset: 0,
    })
    .await
    .unwrap()
    .into_inner();
    assert_eq!(resp.total_count, 2);

    // Filter by priority
    let resp = list_items(centy_daemon::server::proto::ListItemsRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        status: String::new(),
        priority: 1,
        include_deleted: false,
        limit: 0,
        offset: 0,
    })
    .await
    .unwrap()
    .into_inner();
    assert_eq!(resp.total_count, 2);

    // Limit + offset
    let resp = list_items(centy_daemon::server::proto::ListItemsRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        status: String::new(),
        priority: 0,
        include_deleted: false,
        limit: 1,
        offset: 1,
    })
    .await
    .unwrap()
    .into_inner();
    assert_eq!(resp.total_count, 1);
}

// ─── Update item fields ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_update_item() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = create_item(create_req(
        pp,
        "issues",
        "Original",
        "Original body",
        "open",
        2,
        HashMap::new(),
    ))
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success);
    let item_id = resp.item.unwrap().id;

    let resp = update_item(centy_daemon::server::proto::UpdateItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        title: "Updated Title".to_string(),
        body: "Updated body".to_string(),
        status: "closed".to_string(),
        priority: 1,
        custom_fields: HashMap::new(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "update failed: {}", resp.error);
    let updated = resp.item.unwrap();
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.body, "Updated body");
    let meta = updated.metadata.unwrap();
    assert_eq!(meta.status, "closed");
    assert_eq!(meta.priority, 1);
}

// ─── Hard delete (force=true) ────────────────────────────────────────────────

#[tokio::test]
async fn test_hard_delete() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = create_item(create_req(
        pp,
        "issues",
        "To Delete",
        "",
        "open",
        2,
        HashMap::new(),
    ))
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success);
    let item_id = resp.item.unwrap().id;

    let resp = delete_item(centy_daemon::server::proto::DeleteItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
        force: true,
    })
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success, "delete failed: {}", resp.error);

    // Should not be found
    let resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id,
    })
    .await
    .unwrap()
    .into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("ITEM_NOT_FOUND"));
}

// ─── Soft-delete + Restore lifecycle ─────────────────────────────────────────

#[tokio::test]
async fn test_soft_delete_and_restore() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = create_item(create_req(
        pp,
        "issues",
        "Soft Delete Me",
        "",
        "open",
        2,
        HashMap::new(),
    ))
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success);
    let item_id = resp.item.unwrap().id;

    // Soft delete
    let resp = soft_delete_item(centy_daemon::server::proto::SoftDeleteItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
    })
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success, "soft_delete failed: {}", resp.error);
    let meta = resp.item.unwrap().metadata.unwrap();
    assert!(!meta.deleted_at.is_empty(), "deleted_at should be set");

    // Should not appear in regular list
    let resp = list_items(centy_daemon::server::proto::ListItemsRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        status: String::new(),
        priority: 0,
        include_deleted: false,
        limit: 0,
        offset: 0,
    })
    .await
    .unwrap()
    .into_inner();
    assert_eq!(resp.total_count, 0);

    // Should appear with include_deleted
    let resp = list_items(centy_daemon::server::proto::ListItemsRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        status: String::new(),
        priority: 0,
        include_deleted: true,
        limit: 0,
        offset: 0,
    })
    .await
    .unwrap()
    .into_inner();
    assert_eq!(resp.total_count, 1);

    // Restore
    let resp = restore_item(centy_daemon::server::proto::RestoreItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item_id.clone(),
    })
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success, "restore failed: {}", resp.error);
    let meta = resp.item.unwrap().metadata.unwrap();
    assert!(meta.deleted_at.is_empty(), "deleted_at should be cleared");

    // Should appear in regular list again
    let resp = list_items(centy_daemon::server::proto::ListItemsRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        status: String::new(),
        priority: 0,
        include_deleted: false,
        limit: 0,
        offset: 0,
    })
    .await
    .unwrap()
    .into_inner();
    assert_eq!(resp.total_count, 1);
}

// ─── Invalid item_type returns ITEM_TYPE_NOT_FOUND ───────────────────────────

#[tokio::test]
async fn test_invalid_item_type() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = create_item(create_req(
        pp,
        "nonexistent",
        "Test",
        "",
        "",
        0,
        HashMap::new(),
    ))
    .await
    .unwrap()
    .into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("ITEM_TYPE_NOT_FOUND"));

    let resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "nonexistent".to_string(),
        item_id: "some-id".to_string(),
    })
    .await
    .unwrap()
    .into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("ITEM_TYPE_NOT_FOUND"));
}

// ─── Item type normalization ─────────────────────────────────────────────────

#[test]
fn test_item_type_normalization() {
    assert_eq!(normalize_item_type("issue"), "issues");
    assert_eq!(normalize_item_type("issues"), "issues");
    assert_eq!(normalize_item_type("Issue"), "issues");
    assert_eq!(normalize_item_type("ISSUES"), "issues");
    assert_eq!(normalize_item_type("doc"), "docs");
    assert_eq!(normalize_item_type("docs"), "docs");
    assert_eq!(normalize_item_type("Doc"), "docs");
    assert_eq!(normalize_item_type("epics"), "epics");
    assert_eq!(normalize_item_type("custom"), "custom");
}

#[tokio::test]
async fn test_singular_item_type_works() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    // "issue" (singular) should work the same as "issues"
    let resp = create_item(create_req(
        pp,
        "issue",
        "Singular Test",
        "",
        "open",
        2,
        HashMap::new(),
    ))
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success, "create with 'issue' failed: {}", resp.error);
    assert_eq!(resp.item.unwrap().item_type, "issues");
}

// ─── Custom fields roundtrip through proto conversion ────────────────────────

#[tokio::test]
async fn test_custom_fields_roundtrip() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let mut custom_fields = HashMap::new();
    custom_fields.insert("env".to_string(), "\"production\"".to_string());
    custom_fields.insert("count".to_string(), "42".to_string());
    custom_fields.insert("tags".to_string(), "[\"bug\",\"urgent\"]".to_string());

    let resp = create_item(create_req(
        pp,
        "issues",
        "Custom Fields",
        "",
        "open",
        2,
        custom_fields,
    ))
    .await
    .unwrap()
    .into_inner();
    assert!(resp.success, "create failed: {}", resp.error);
    let item = resp.item.unwrap();
    let meta = item.metadata.unwrap();

    // Custom fields should be preserved as JSON strings
    assert_eq!(meta.custom_fields.get("env").unwrap(), "\"production\"");
    assert_eq!(meta.custom_fields.get("count").unwrap(), "42");
    assert_eq!(
        meta.custom_fields.get("tags").unwrap(),
        "[\"bug\",\"urgent\"]"
    );

    // Verify they survive a get roundtrip
    let get_resp = get_item(centy_daemon::server::proto::GetItemRequest {
        project_path: pp.to_string(),
        item_type: "issues".to_string(),
        item_id: item.id,
    })
    .await
    .unwrap()
    .into_inner();
    assert!(get_resp.success);
    let fetched_meta = get_resp.item.unwrap().metadata.unwrap();
    assert_eq!(
        fetched_meta.custom_fields.get("env").unwrap(),
        "\"production\""
    );
    assert_eq!(fetched_meta.custom_fields.get("count").unwrap(), "42");
}
