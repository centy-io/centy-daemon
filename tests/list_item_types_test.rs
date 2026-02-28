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

use centy_daemon::server::handlers::item_type_create::create_item_type;
use centy_daemon::server::handlers::item_type_list::list_item_types;
use centy_daemon::server::proto::{
    CreateItemTypeRequest, ItemTypeFeatures as ProtoFeatures, ListItemTypesRequest,
};
use common::{create_test_dir, init_centy_project};

// ─── Success: initialized project returns built-in item types ────────────────

#[tokio::test]
async fn test_list_item_types_initialized_project() {
    let temp = create_test_dir();
    let path = temp.path();
    init_centy_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = list_item_types(ListItemTypesRequest {
        project_path: pp.to_string(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "list failed: {}", resp.error);
    assert!(
        resp.total_count >= 2,
        "expected at least 2 built-in types, got {}",
        resp.total_count
    );
    assert_eq!(resp.item_types.len() as i32, resp.total_count);

    let plurals: Vec<&str> = resp.item_types.iter().map(|t| t.plural.as_str()).collect();
    assert!(plurals.contains(&"issues"), "should include issues type");
    assert!(plurals.contains(&"docs"), "should include docs type");
}

// ─── Success: total_count matches item_types length ──────────────────────────

#[tokio::test]
async fn test_list_item_types_total_count_matches() {
    let temp = create_test_dir();
    let path = temp.path();
    init_centy_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = list_item_types(ListItemTypesRequest {
        project_path: pp.to_string(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success);
    assert_eq!(resp.item_types.len() as i32, resp.total_count);
}

// ─── Success: custom type appears in list ────────────────────────────────────

#[tokio::test]
async fn test_list_item_types_includes_custom_type() {
    let temp = create_test_dir();
    let path = temp.path();
    init_centy_project(path).await;
    let pp = path.to_str().unwrap();

    // Create a custom type
    let create_resp = create_item_type(CreateItemTypeRequest {
        project_path: pp.to_string(),
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
        statuses: vec!["open".to_string(), "closed".to_string()],
        default_status: "open".to_string(),
        priority_levels: 2,
        custom_fields: vec![],
    })
    .await
    .unwrap()
    .into_inner();
    assert!(create_resp.success, "create failed: {}", create_resp.error);

    let resp = list_item_types(ListItemTypesRequest {
        project_path: pp.to_string(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "list failed: {}", resp.error);

    let bug_type = resp.item_types.iter().find(|t| t.plural == "bugs");
    assert!(
        bug_type.is_some(),
        "custom 'bugs' type should appear in list"
    );
    let bug = bug_type.unwrap();
    assert_eq!(bug.name, "Bug");
    assert_eq!(bug.plural, "bugs");
    assert_eq!(bug.identifier, "uuid");
}

// ─── Success: empty project (no item types configured) ───────────────────────

#[tokio::test]
async fn test_list_item_types_empty_project() {
    let temp = create_test_dir();
    let path = temp.path();

    // Only create .centy dir without any item type sub-directories
    tokio::fs::create_dir_all(path.join(".centy"))
        .await
        .unwrap();

    let pp = path.to_str().unwrap();

    let resp = list_item_types(ListItemTypesRequest {
        project_path: pp.to_string(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "list failed: {}", resp.error);
    assert_eq!(resp.total_count, 0);
    assert!(resp.item_types.is_empty());
}

// ─── ItemTypeConfigProto fields are populated ────────────────────────────────

#[tokio::test]
async fn test_list_item_types_config_fields_populated() {
    let temp = create_test_dir();
    let path = temp.path();
    init_centy_project(path).await;
    let pp = path.to_str().unwrap();

    let resp = list_item_types(ListItemTypesRequest {
        project_path: pp.to_string(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success);
    let issues = resp
        .item_types
        .iter()
        .find(|t| t.plural == "issues")
        .unwrap();
    assert!(!issues.name.is_empty(), "name should be populated");
    assert!(!issues.plural.is_empty(), "plural should be populated");
    assert!(
        !issues.identifier.is_empty(),
        "identifier should be populated"
    );
    assert!(issues.features.is_some(), "features should be populated");
}
