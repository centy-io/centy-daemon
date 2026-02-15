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

use centy_daemon::config::item_type_config::{
    default_doc_config, default_issue_config, discover_item_types, read_item_type_config,
    ItemTypeConfig, ItemTypeFeatures,
};
use centy_daemon::config::CentyConfig;
use centy_daemon::item::core::crud::ItemFilters;
use centy_daemon::item::core::error::ItemError;
use centy_daemon::item::generic::reconcile::{
    get_next_display_number_generic, reconcile_display_numbers_generic,
};
use centy_daemon::item::generic::storage::{
    generic_create, generic_delete, generic_get, generic_list, generic_restore,
    generic_soft_delete, generic_update,
};
use centy_daemon::item::generic::types::{
    CreateGenericItemOptions, GenericFrontmatter, UpdateGenericItemOptions,
};
use common::create_test_dir;
use std::collections::HashMap;
use tokio::fs;

/// Initialize a minimal project for generic storage tests.
async fn init_generic_project(project_path: &std::path::Path) {
    let centy_path = project_path.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();

    // Write a minimal manifest
    let manifest = centy_daemon::manifest::create_manifest();
    centy_daemon::manifest::write_manifest(project_path, &manifest)
        .await
        .unwrap();
}

/// Helper to create a minimal config with no features enabled.
fn minimal_config() -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Note".to_string(),
        plural: "notes".to_string(),
        identifier: "uuid".to_string(),
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    }
}

// ─── Full CRUD Roundtrip (all features enabled) ─────────────────────────────

#[tokio::test]
async fn test_full_crud_roundtrip() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default());

    // Create
    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Full CRUD Test".to_string(),
            body: "Testing all features.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(1),
            custom_fields: HashMap::from([("env".to_string(), serde_json::json!("prod"))]),
        },
    )
    .await
    .unwrap();

    assert_eq!(item.title, "Full CRUD Test");
    assert_eq!(item.body, "Testing all features.");
    assert_eq!(item.frontmatter.display_number, Some(1));
    assert_eq!(item.frontmatter.status, Some("open".to_string()));
    assert_eq!(item.frontmatter.priority, Some(1));
    assert_eq!(
        item.frontmatter.custom_fields.get("env"),
        Some(&serde_json::json!("prod"))
    );
    assert!(item.frontmatter.deleted_at.is_none());

    // Read
    let fetched = generic_get(path, &config, &item.id).await.unwrap();
    assert_eq!(fetched.title, "Full CRUD Test");
    assert_eq!(fetched.frontmatter.display_number, Some(1));

    // Update
    let updated = generic_update(
        path,
        &config,
        &item.id,
        UpdateGenericItemOptions {
            title: Some("Updated Title".to_string()),
            body: Some("Updated body.".to_string()),
            status: Some("closed".to_string()),
            priority: Some(3),
            custom_fields: HashMap::from([("team".to_string(), serde_json::json!("backend"))]),
        },
    )
    .await
    .unwrap();

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.body, "Updated body.");
    assert_eq!(updated.frontmatter.status, Some("closed".to_string()));
    assert_eq!(updated.frontmatter.priority, Some(3));
    // Original custom field preserved
    assert_eq!(
        updated.frontmatter.custom_fields.get("env"),
        Some(&serde_json::json!("prod"))
    );
    // New custom field added
    assert_eq!(
        updated.frontmatter.custom_fields.get("team"),
        Some(&serde_json::json!("backend"))
    );

    // List
    let items = generic_list(path, &config, ItemFilters::default())
        .await
        .unwrap();
    assert_eq!(items.len(), 1);

    // Hard delete
    generic_delete(path, &config, &item.id, true).await.unwrap();
    let result = generic_get(path, &config, &item.id).await;
    assert!(matches!(result, Err(ItemError::NotFound(_))));
}

// ─── CRUD with minimal features ─────────────────────────────────────────────

#[tokio::test]
async fn test_crud_minimal_features() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = minimal_config();

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Simple Note".to_string(),
            body: "Just a note.".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    // No optional features should be set
    assert!(item.frontmatter.display_number.is_none());
    assert!(item.frontmatter.status.is_none());
    assert!(item.frontmatter.priority.is_none());
    assert!(item.frontmatter.deleted_at.is_none());

    // Get it back
    let fetched = generic_get(path, &config, &item.id).await.unwrap();
    assert_eq!(fetched.title, "Simple Note");
    assert!(fetched.frontmatter.display_number.is_none());
}

// ─── Display number auto-increment ──────────────────────────────────────────

#[tokio::test]
async fn test_display_number_auto_increment() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default());

    for i in 1..=5u32 {
        let item = generic_create(
            path,
            &config,
            CreateGenericItemOptions {
                title: format!("Issue {i}"),
                body: String::new(),
                id: None,
                status: Some("open".to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            },
        )
        .await
        .unwrap();
        assert_eq!(item.frontmatter.display_number, Some(i));
    }

    // Verify next display number
    let storage_path = path.join(".centy").join("issues");
    let next = get_next_display_number_generic(&storage_path)
        .await
        .unwrap();
    assert_eq!(next, 6);
}

// ─── Status validation ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_status_validation_on_create() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default());

    // Valid status
    let result = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Good Status".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        },
    )
    .await;
    assert!(result.is_ok());

    // Invalid status
    let result = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Bad Status".to_string(),
            body: String::new(),
            id: None,
            status: Some("invalid-status".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        },
    )
    .await;
    assert!(matches!(result, Err(ItemError::InvalidStatus { .. })));
}

#[tokio::test]
async fn test_status_validation_on_update() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default());

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Status Update Test".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    // Valid status update
    let result = generic_update(
        path,
        &config,
        &item.id,
        UpdateGenericItemOptions {
            status: Some("closed".to_string()),
            ..Default::default()
        },
    )
    .await;
    assert!(result.is_ok());

    // Invalid status update
    let result = generic_update(
        path,
        &config,
        &item.id,
        UpdateGenericItemOptions {
            status: Some("nonexistent".to_string()),
            ..Default::default()
        },
    )
    .await;
    assert!(matches!(result, Err(ItemError::InvalidStatus { .. })));
}

// ─── Priority validation ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_priority_validation() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default()); // priority_levels = 3

    // Valid priority (1-3)
    for p in 1..=3u32 {
        let result = generic_create(
            path,
            &config,
            CreateGenericItemOptions {
                title: format!("Priority {p}"),
                body: String::new(),
                id: None,
                status: Some("open".to_string()),
                priority: Some(p),
                custom_fields: HashMap::new(),
            },
        )
        .await;
        assert!(result.is_ok());
    }

    // Invalid priority
    let result = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Bad Priority".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(99),
            custom_fields: HashMap::new(),
        },
    )
    .await;
    assert!(matches!(result, Err(ItemError::InvalidPriority { .. })));

    // Priority 0 is also invalid
    let result = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Zero Priority".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(0),
            custom_fields: HashMap::new(),
        },
    )
    .await;
    assert!(matches!(result, Err(ItemError::InvalidPriority { .. })));
}

// ─── Soft delete / restore / hard delete ─────────────────────────────────────

#[tokio::test]
async fn test_soft_delete_restore_hard_delete() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default());

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Delete Me".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    // Soft delete
    generic_soft_delete(path, &config, &item.id).await.unwrap();

    // Should not appear in regular list
    let items = generic_list(path, &config, ItemFilters::default())
        .await
        .unwrap();
    assert!(items.is_empty());

    // Should appear with include_deleted
    let items = generic_list(path, &config, ItemFilters::new().include_deleted())
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].frontmatter.deleted_at.is_some());

    // Cannot soft-delete again
    let result = generic_soft_delete(path, &config, &item.id).await;
    assert!(matches!(result, Err(ItemError::IsDeleted(_))));

    // Cannot update a deleted item
    let result = generic_update(
        path,
        &config,
        &item.id,
        UpdateGenericItemOptions {
            title: Some("Nope".to_string()),
            ..Default::default()
        },
    )
    .await;
    assert!(matches!(result, Err(ItemError::IsDeleted(_))));

    // Restore
    generic_restore(path, &config, &item.id).await.unwrap();

    // Should appear again
    let items = generic_list(path, &config, ItemFilters::default())
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].frontmatter.deleted_at.is_none());

    // Hard delete
    generic_delete(path, &config, &item.id, true).await.unwrap();
    let result = generic_get(path, &config, &item.id).await;
    assert!(matches!(result, Err(ItemError::NotFound(_))));
}

// ─── List with filters ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_filters() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default());

    // Create items with different statuses and priorities
    let combinations = [
        ("Open P1", "open", 1u32),
        ("Open P2", "open", 2),
        ("Open P3", "open", 3),
        ("Closed P1", "closed", 1),
        ("Closed P2", "closed", 2),
    ];

    for (title, status, priority) in combinations {
        generic_create(
            path,
            &config,
            CreateGenericItemOptions {
                title: title.to_string(),
                body: String::new(),
                id: None,
                status: Some(status.to_string()),
                priority: Some(priority),
                custom_fields: HashMap::new(),
            },
        )
        .await
        .unwrap();
    }

    // Filter by status
    let open = generic_list(path, &config, ItemFilters::new().with_status("open"))
        .await
        .unwrap();
    assert_eq!(open.len(), 3);

    let closed = generic_list(path, &config, ItemFilters::new().with_status("closed"))
        .await
        .unwrap();
    assert_eq!(closed.len(), 2);

    // Filter by priority
    let p1 = generic_list(path, &config, ItemFilters::new().with_priority(1))
        .await
        .unwrap();
    assert_eq!(p1.len(), 2);

    // Combined filters
    let open_p1 = generic_list(
        path,
        &config,
        ItemFilters::new().with_status("open").with_priority(1),
    )
    .await
    .unwrap();
    assert_eq!(open_p1.len(), 1);

    // Limit
    let limited = generic_list(path, &config, ItemFilters::new().with_limit(2))
        .await
        .unwrap();
    assert_eq!(limited.len(), 2);

    // Offset
    let offset = generic_list(path, &config, ItemFilters::new().with_offset(3))
        .await
        .unwrap();
    assert_eq!(offset.len(), 2);

    // Offset + Limit
    let paged = generic_list(
        path,
        &config,
        ItemFilters::new().with_offset(1).with_limit(2),
    )
    .await
    .unwrap();
    assert_eq!(paged.len(), 2);
}

// ─── UUID vs Slug ID strategies ─────────────────────────────────────────────

#[tokio::test]
async fn test_uuid_id_strategy() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default()); // UUID strategy

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "UUID Item".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    // ID should be UUID format (8-4-4-4-12)
    let parts: Vec<&str> = item.id.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
}

#[tokio::test]
async fn test_slug_id_strategy() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_doc_config();

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Getting Started Guide".to_string(),
            body: "Welcome to the docs!".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    assert_eq!(item.id, "getting-started-guide");

    // Get by slug
    let fetched = generic_get(path, &config, "getting-started-guide")
        .await
        .unwrap();
    assert_eq!(fetched.title, "Getting Started Guide");
}

#[tokio::test]
async fn test_slug_strategy_duplicate_id() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_doc_config();

    // First create succeeds
    generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Same Title".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    // Second create with same title should fail (same slug)
    let result = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Same Title".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await;
    assert!(matches!(result, Err(ItemError::AlreadyExists(_))));
}

#[tokio::test]
async fn test_explicit_id() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = minimal_config();

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Explicit ID".to_string(),
            body: String::new(),
            id: Some("my-custom-id".to_string()),
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    assert_eq!(item.id, "my-custom-id");

    let fetched = generic_get(path, &config, "my-custom-id").await.unwrap();
    assert_eq!(fetched.title, "Explicit ID");
}

// ─── Config discovery ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_config_discovery() {
    let temp = create_test_dir();
    let path = temp.path();
    let centy_path = path.join(".centy");

    // Create configs for two types
    let issues_path = centy_path.join("issues");
    fs::create_dir_all(&issues_path).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_path.join("config.yaml"), &yaml)
        .await
        .unwrap();

    let docs_path = centy_path.join("docs");
    fs::create_dir_all(&docs_path).await.unwrap();
    let doc_config = default_doc_config();
    let yaml = serde_yaml::to_string(&doc_config).unwrap();
    fs::write(docs_path.join("config.yaml"), &yaml)
        .await
        .unwrap();

    // Create a folder without config (should be skipped)
    fs::create_dir_all(centy_path.join("assets")).await.unwrap();

    let configs = discover_item_types(path).await.unwrap();
    assert_eq!(configs.len(), 2);

    let names: Vec<String> = configs.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"Issue".to_string()));
    assert!(names.contains(&"Doc".to_string()));
}

#[tokio::test]
async fn test_read_config() {
    let temp = create_test_dir();
    let path = temp.path();
    let centy_path = path.join(".centy");

    let epics_path = centy_path.join("epics");
    fs::create_dir_all(&epics_path).await.unwrap();

    let config = ItemTypeConfig {
        name: "Epic".to_string(),
        plural: "epics".to_string(),
        identifier: "uuid".to_string(),
        features: ItemTypeFeatures {
            display_number: true,
            status: true,
            priority: false,
            assets: false,
            org_sync: false,
            move_item: false,
            duplicate: false,
        },
        statuses: vec!["open".to_string(), "closed".to_string()],
        default_status: Some("open".to_string()),
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    let yaml = serde_yaml::to_string(&config).unwrap();
    fs::write(epics_path.join("config.yaml"), &yaml)
        .await
        .unwrap();

    let loaded = read_item_type_config(path, "epics").await.unwrap().unwrap();
    assert_eq!(loaded.name, "Epic");
    assert!(loaded.features.display_number);
    assert!(!loaded.features.priority);
}

#[tokio::test]
async fn test_read_config_not_found() {
    let temp = create_test_dir();
    let result = read_item_type_config(temp.path(), "nonexistent")
        .await
        .unwrap();
    assert!(result.is_none());
}

// ─── Reconciliation ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_reconcile_display_numbers_no_conflicts() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default());

    // Create items with sequential display numbers
    for i in 1..=3u32 {
        generic_create(
            path,
            &config,
            CreateGenericItemOptions {
                title: format!("Issue {i}"),
                body: String::new(),
                id: None,
                status: Some("open".to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            },
        )
        .await
        .unwrap();
    }

    let storage_path = path.join(".centy").join("issues");
    let reassigned = reconcile_display_numbers_generic(&storage_path)
        .await
        .unwrap();
    assert_eq!(reassigned, 0);
}

#[tokio::test]
async fn test_reconcile_display_numbers_with_conflicts() {
    let temp = create_test_dir();
    let storage_path = temp.path().join("items");
    fs::create_dir_all(&storage_path).await.unwrap();

    // Manually write items with conflicting display numbers
    let fm1 = GenericFrontmatter {
        display_number: Some(1),
        status: Some("open".to_string()),
        priority: Some(2),
        created_at: "2024-01-01T10:00:00Z".to_string(),
        updated_at: "2024-01-01T10:00:00Z".to_string(),
        deleted_at: None,
        custom_fields: HashMap::new(),
    };
    let content1 = centy_daemon::common::generate_frontmatter(&fm1, "Item A", "");
    fs::write(storage_path.join("item-a.md"), &content1)
        .await
        .unwrap();

    // Same display_number but newer
    let fm2 = GenericFrontmatter {
        display_number: Some(1),
        status: Some("open".to_string()),
        priority: Some(2),
        created_at: "2024-01-01T11:00:00Z".to_string(),
        updated_at: "2024-01-01T11:00:00Z".to_string(),
        deleted_at: None,
        custom_fields: HashMap::new(),
    };
    let content2 = centy_daemon::common::generate_frontmatter(&fm2, "Item B", "");
    fs::write(storage_path.join("item-b.md"), &content2)
        .await
        .unwrap();

    let fm3 = GenericFrontmatter {
        display_number: Some(2),
        status: Some("open".to_string()),
        priority: Some(2),
        created_at: "2024-01-01T12:00:00Z".to_string(),
        updated_at: "2024-01-01T12:00:00Z".to_string(),
        deleted_at: None,
        custom_fields: HashMap::new(),
    };
    let content3 = centy_daemon::common::generate_frontmatter(&fm3, "Item C", "");
    fs::write(storage_path.join("item-c.md"), &content3)
        .await
        .unwrap();

    let reassigned = reconcile_display_numbers_generic(&storage_path)
        .await
        .unwrap();
    assert_eq!(reassigned, 1);

    // Item A (older) should keep display_number 1
    let content = fs::read_to_string(storage_path.join("item-a.md"))
        .await
        .unwrap();
    let (fm, _, _) =
        centy_daemon::common::parse_frontmatter::<GenericFrontmatter>(&content).unwrap();
    assert_eq!(fm.display_number, Some(1));

    // Item B (newer) should get display_number 3 (max was 2, next is 3)
    let content = fs::read_to_string(storage_path.join("item-b.md"))
        .await
        .unwrap();
    let (fm, _, _) =
        centy_daemon::common::parse_frontmatter::<GenericFrontmatter>(&content).unwrap();
    assert_eq!(fm.display_number, Some(3));
}

// ─── Default status assignment ──────────────────────────────────────────────

#[tokio::test]
async fn test_default_status_when_not_provided() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default()); // default_status = "open"

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Default Status".to_string(),
            body: String::new(),
            id: None,
            status: None, // Should use default
            priority: Some(2),
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    assert_eq!(item.frontmatter.status, Some("open".to_string()));
}

#[tokio::test]
async fn test_default_priority_when_not_provided() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default()); // priority_levels = 3

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Default Priority".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: None, // Should use default (middle = 2 for 3 levels)
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    assert_eq!(item.frontmatter.priority, Some(2));
}

// ─── Custom item type (Epic) ────────────────────────────────────────────────

#[tokio::test]
async fn test_custom_epic_type() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = ItemTypeConfig {
        name: "Epic".to_string(),
        plural: "epics".to_string(),
        identifier: "uuid".to_string(),
        features: ItemTypeFeatures {
            display_number: true,
            status: true,
            priority: false, // No priority for epics
            assets: false,
            org_sync: false,
            move_item: false,
            duplicate: false,
        },
        statuses: vec![
            "backlog".to_string(),
            "active".to_string(),
            "done".to_string(),
        ],
        default_status: Some("backlog".to_string()),
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "User Authentication".to_string(),
            body: "Implement complete auth flow.".to_string(),
            id: None,
            status: Some("active".to_string()),
            priority: None, // Priority disabled
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    assert_eq!(item.item_type, "epics");
    assert_eq!(item.frontmatter.display_number, Some(1));
    assert_eq!(item.frontmatter.status, Some("active".to_string()));
    assert!(item.frontmatter.priority.is_none());

    // Verify file is in correct directory
    let file_path = path
        .join(".centy")
        .join("epics")
        .join(format!("{}.md", item.id));
    assert!(file_path.exists());
}

// ─── List ordering ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_ordered_by_display_number() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = default_issue_config(&CentyConfig::default());

    // Create in reverse order
    for title in ["Third", "Second", "First"] {
        generic_create(
            path,
            &config,
            CreateGenericItemOptions {
                title: title.to_string(),
                body: String::new(),
                id: None,
                status: Some("open".to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            },
        )
        .await
        .unwrap();
    }

    let items = generic_list(path, &config, ItemFilters::default())
        .await
        .unwrap();
    assert_eq!(items.len(), 3);
    // Should be ordered by display_number
    assert_eq!(items[0].frontmatter.display_number, Some(1));
    assert_eq!(items[1].frontmatter.display_number, Some(2));
    assert_eq!(items[2].frontmatter.display_number, Some(3));
}

// ─── Delete without force always soft-deletes first ──────────────────────────

#[tokio::test]
async fn test_delete_without_force_soft_deletes_first() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let config = minimal_config();

    let item = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "Soft first".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await
    .unwrap();

    // delete with force=false should soft delete
    generic_delete(path, &config, &item.id, false)
        .await
        .unwrap();

    // Item should still exist but be soft-deleted
    let fetched = generic_get(path, &config, &item.id).await.unwrap();
    assert!(fetched.frontmatter.deleted_at.is_some());
}

// ─── Empty slug from empty title ────────────────────────────────────────────

#[tokio::test]
async fn test_slug_from_empty_title_fails() {
    let temp = create_test_dir();
    let path = temp.path();
    init_generic_project(path).await;

    let mut config = minimal_config();
    config.identifier = "slug".to_string();

    let result = generic_create(
        path,
        &config,
        CreateGenericItemOptions {
            title: "   ".to_string(), // Only spaces -> empty slug
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await;
    assert!(matches!(result, Err(ItemError::ValidationError(_))));
}
