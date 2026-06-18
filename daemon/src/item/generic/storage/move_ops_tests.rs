//! Tests for `generic_duplicate` and `generic_rename_slug`.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::move_ops::{generic_duplicate, generic_rename_slug};
use crate::item::core::error::ItemError;
use crate::item::generic::storage::generic_create;
use crate::item::generic::types::DuplicateGenericItemOptions;
use crate::manifest;
use mdstore::{CreateOptions, TypeConfig};
use std::collections::HashMap;
use tokio::fs;

async fn setup_project(temp: &std::path::Path) {
    let centy_path = temp.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let m = manifest::create_manifest();
    manifest::write_manifest(temp, &m).await.unwrap();
}

fn minimal_config() -> TypeConfig {
    TypeConfig {
        name: "Note".to_string(),
        identifier: mdstore::IdStrategy::Uuid,
        features: mdstore::TypeFeatures {
            duplicate: true,
            ..Default::default()
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    }
}

fn slug_config() -> TypeConfig {
    TypeConfig {
        name: "Doc".to_string(),
        identifier: mdstore::IdStrategy::Slug,
        features: mdstore::TypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    }
}

// ─── generic_duplicate ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_generic_duplicate_same_project() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let config = minimal_config();

    // Create an item first
    let created = generic_create(
        temp.path(),
        "notes",
        &config,
        CreateOptions {
            title: "Original Note".to_string(),
            body: "Original body.".to_string(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    // Duplicate it
    let options = DuplicateGenericItemOptions {
        source_project_path: temp.path().to_path_buf(),
        target_project_path: temp.path().to_path_buf(),
        item_id: created.id.clone(),
        new_id: None,
        new_title: None,
    };
    let result = generic_duplicate("notes", &config, options).await.unwrap();

    assert_eq!(result.original_id, created.id);
    assert_ne!(result.item.id, created.id);
    // Default title should contain "Copy of"
    assert!(result.item.title.contains("Copy of") || result.item.title.contains("Original Note"));
}

#[tokio::test]
async fn test_generic_duplicate_with_custom_title() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let config = minimal_config();

    let created = generic_create(
        temp.path(),
        "notes",
        &config,
        CreateOptions {
            title: "My Note".to_string(),
            body: "Content.".to_string(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    let options = DuplicateGenericItemOptions {
        source_project_path: temp.path().to_path_buf(),
        target_project_path: temp.path().to_path_buf(),
        item_id: created.id.clone(),
        new_id: None,
        new_title: Some("Custom Title".to_string()),
    };
    let result = generic_duplicate("notes", &config, options).await.unwrap();
    assert_eq!(result.item.title, "Custom Title");
}

#[tokio::test]
async fn test_generic_duplicate_cross_project() {
    let src_temp = tempfile::tempdir().unwrap();
    let tgt_temp = tempfile::tempdir().unwrap();
    setup_project(src_temp.path()).await;
    setup_project(tgt_temp.path()).await;
    let config = minimal_config();

    let created = generic_create(
        src_temp.path(),
        "notes",
        &config,
        CreateOptions {
            title: "Cross Project Note".to_string(),
            body: "Move me.".to_string(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    let options = DuplicateGenericItemOptions {
        source_project_path: src_temp.path().to_path_buf(),
        target_project_path: tgt_temp.path().to_path_buf(),
        item_id: created.id.clone(),
        new_id: None,
        new_title: None,
    };
    let result = generic_duplicate("notes", &config, options).await.unwrap();
    assert_eq!(result.original_id, created.id);

    // Verify item exists in the target project
    let target_file = tgt_temp
        .path()
        .join(".centy")
        .join("notes")
        .join(format!("{}.md", result.item.id));
    assert!(
        target_file.exists(),
        "Duplicated item should exist in target project"
    );
}

#[tokio::test]
async fn test_generic_duplicate_with_assets() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let mut config = minimal_config();
    config.features.assets = true;

    let created = generic_create(
        temp.path(),
        "items",
        &config,
        CreateOptions {
            title: "Item With Assets".to_string(),
            body: "Has files.".to_string(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    // Create a legacy-style asset for this item
    let assets_dir = temp
        .path()
        .join(".centy")
        .join("items")
        .join("assets")
        .join(&created.id);
    fs::create_dir_all(&assets_dir).await.unwrap();
    fs::write(assets_dir.join("asset.txt"), b"asset content")
        .await
        .unwrap();

    let options = DuplicateGenericItemOptions {
        source_project_path: temp.path().to_path_buf(),
        target_project_path: temp.path().to_path_buf(),
        item_id: created.id.clone(),
        new_id: None,
        new_title: None,
    };
    let result = generic_duplicate("items", &config, options).await.unwrap();

    // The new item's assets should also have been copied
    let target_assets = temp
        .path()
        .join(".centy")
        .join("items")
        .join("assets")
        .join(&result.item.id);
    assert!(
        target_assets.join("asset.txt").exists(),
        "Assets should be copied to duplicate"
    );
}

#[tokio::test]
async fn test_generic_duplicate_assets_no_source_assets() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let mut config = minimal_config();
    config.features.assets = true;

    let created = generic_create(
        temp.path(),
        "items",
        &config,
        CreateOptions {
            title: "Item Without Assets".to_string(),
            body: "No files.".to_string(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    // No asset directory created — should still succeed
    let options = DuplicateGenericItemOptions {
        source_project_path: temp.path().to_path_buf(),
        target_project_path: temp.path().to_path_buf(),
        item_id: created.id.clone(),
        new_id: None,
        new_title: None,
    };
    let result = generic_duplicate("items", &config, options).await.unwrap();
    assert_eq!(result.original_id, created.id);
}

// ─── generic_rename_slug ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_generic_rename_slug_success() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let config = slug_config();

    let created = generic_create(
        temp.path(),
        "docs",
        &config,
        CreateOptions {
            title: "Getting Started".to_string(),
            body: "Guide content.".to_string(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(created.id, "getting-started");

    let result = generic_rename_slug(temp.path(), "docs", &config, "getting-started", "new-guide")
        .await
        .unwrap();

    assert_eq!(result.old_id, "getting-started");
    assert_eq!(result.item.id, "new-guide");

    // Old file should be gone
    let old_file = temp
        .path()
        .join(".centy")
        .join("docs")
        .join("getting-started.md");
    assert!(!old_file.exists(), "Old file should not exist after rename");

    // New file should exist
    let new_file = temp.path().join(".centy").join("docs").join("new-guide.md");
    assert!(new_file.exists(), "New file should exist after rename");
}

#[tokio::test]
async fn test_generic_rename_slug_source_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let config = slug_config();

    // Create the docs directory
    fs::create_dir_all(temp.path().join(".centy").join("docs"))
        .await
        .unwrap();

    let result = generic_rename_slug(temp.path(), "docs", &config, "nonexistent", "new-id").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ItemError::NotFound(_)));
}

#[tokio::test]
async fn test_generic_rename_slug_target_already_exists() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let config = slug_config();

    // Create both items
    generic_create(
        temp.path(),
        "docs",
        &config,
        CreateOptions {
            title: "First Doc".to_string(),
            body: "First.".to_string(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    generic_create(
        temp.path(),
        "docs",
        &config,
        CreateOptions {
            title: "Second Doc".to_string(),
            body: "Second.".to_string(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    // Try to rename first to second
    let result = generic_rename_slug(temp.path(), "docs", &config, "first-doc", "second-doc").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ItemError::Custom(_)));
}
