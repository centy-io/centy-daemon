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
use centy_daemon::server::handlers::item_type_create::create_item_type;
use centy_daemon::server::proto::{
    CreateItemRequest, CreateItemTypeRequest, ItemTypeFeatures as ProtoFeatures,
};
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

fn make_request(project_path: &str) -> CreateItemTypeRequest {
    CreateItemTypeRequest {
        project_path: project_path.to_string(),
        name: "Bug".to_string(),
        plural: "bugs".to_string(),
        identifier: "uuid".to_string(),
        features: Some(ProtoFeatures {
            display_number: true,
            status: true,
            priority: true,
            soft_delete: false,
            assets: false,
            org_sync: false,
            r#move: true,
            duplicate: true,
        }),
        statuses: vec![
            "open".to_string(),
            "in-progress".to_string(),
            "closed".to_string(),
        ],
        default_status: "open".to_string(),
        priority_levels: 3,
        custom_fields: vec![],
    }
}

// ─── Success: Create a custom item type ──────────────────────────────────────

#[tokio::test]
async fn test_create_item_type_success() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = create_item_type(make_request(pp))
        .await
        .unwrap()
        .into_inner();

    assert!(resp.success, "create failed: {}", resp.error);
    let config = resp.config.unwrap();
    assert_eq!(config.name, "Bug");
    assert_eq!(config.plural, "bugs");
    assert_eq!(config.identifier, "uuid");
    assert_eq!(config.statuses, vec!["open", "in-progress", "closed"]);
    assert_eq!(config.default_status, "open");
    assert_eq!(config.priority_levels, 3);

    let f = config.features.unwrap();
    assert!(f.display_number);
    assert!(f.status);
    assert!(f.priority);
    assert!(!f.assets);
    assert!(f.r#move);
    assert!(f.duplicate);

    // Verify config.yaml was written to disk
    let config_path = path.join(".centy").join("bugs").join("config.yaml");
    assert!(config_path.exists(), "config.yaml should exist on disk");
}

// ─── After creating type, CreateItem should work with it ─────────────────────

#[tokio::test]
async fn test_create_item_with_custom_type() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    // Create the custom type first
    let type_resp = create_item_type(make_request(pp))
        .await
        .unwrap()
        .into_inner();
    assert!(type_resp.success, "type create failed: {}", type_resp.error);

    // Now create an item of that type
    let item_resp = create_item(CreateItemRequest {
        project_path: pp.to_string(),
        item_type: "bugs".to_string(),
        title: "My First Bug".to_string(),
        body: "Bug description".to_string(),
        status: "open".to_string(),
        priority: 1,
        custom_fields: HashMap::new(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(item_resp.success, "item create failed: {}", item_resp.error);
    let item = item_resp.item.unwrap();
    assert_eq!(item.item_type, "bugs");
    assert_eq!(item.title, "My First Bug");
}

// ─── Validation: empty name ──────────────────────────────────────────────────

#[tokio::test]
async fn test_create_item_type_empty_name() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let mut req = make_request(pp);
    req.name = String::new();

    let resp = create_item_type(req).await.unwrap().into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("VALIDATION_ERROR"));
}

// ─── Validation: invalid plural ──────────────────────────────────────────────

#[tokio::test]
async fn test_create_item_type_invalid_plural() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let mut req = make_request(pp);
    req.plural = "My Bugs".to_string(); // spaces + uppercase

    let resp = create_item_type(req).await.unwrap().into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("VALIDATION_ERROR"));
}

// ─── Validation: invalid identifier ──────────────────────────────────────────

#[tokio::test]
async fn test_create_item_type_invalid_identifier() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let mut req = make_request(pp);
    req.identifier = "number".to_string();

    let resp = create_item_type(req).await.unwrap().into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("VALIDATION_ERROR"));
}

// ─── Validation: default_status not in statuses ──────────────────────────────

#[tokio::test]
async fn test_create_item_type_invalid_default_status() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let mut req = make_request(pp);
    req.default_status = "nonexistent".to_string();

    let resp = create_item_type(req).await.unwrap().into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("VALIDATION_ERROR"));
}

// ─── Duplicate: same plural ──────────────────────────────────────────────────

#[tokio::test]
async fn test_create_item_type_duplicate_plural() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    // First creation succeeds
    let resp1 = create_item_type(make_request(pp))
        .await
        .unwrap()
        .into_inner();
    assert!(resp1.success);

    // Second creation with same plural fails
    let resp2 = create_item_type(make_request(pp))
        .await
        .unwrap()
        .into_inner();
    assert!(!resp2.success);
    assert!(resp2.error.contains("ALREADY_EXISTS"));
}

// ─── Duplicate: same name, different plural ──────────────────────────────────

#[tokio::test]
async fn test_create_item_type_duplicate_name() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let resp1 = create_item_type(make_request(pp))
        .await
        .unwrap()
        .into_inner();
    assert!(resp1.success);

    // Same name "Bug" but different plural "defects"
    let mut req2 = make_request(pp);
    req2.plural = "defects".to_string();

    let resp2 = create_item_type(req2).await.unwrap().into_inner();
    assert!(!resp2.success);
    assert!(resp2.error.contains("ALREADY_EXISTS"));
}

// ─── Slug identifier type ────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_item_type_slug_identifier() {
    let temp = create_test_dir();
    let path = temp.path();
    init_project(path).await;
    let pp = path.to_str().unwrap();

    let mut req = make_request(pp);
    req.name = "Wiki".to_string();
    req.plural = "wiki-pages".to_string();
    req.identifier = "slug".to_string();
    req.statuses = vec![];
    req.default_status = String::new();
    req.priority_levels = 0;

    let resp = create_item_type(req).await.unwrap().into_inner();
    assert!(resp.success, "create failed: {}", resp.error);
    let config = resp.config.unwrap();
    assert_eq!(config.identifier, "slug");
    assert!(config.statuses.is_empty());
    assert!(config.default_status.is_empty());
    assert_eq!(config.priority_levels, 0);
}
