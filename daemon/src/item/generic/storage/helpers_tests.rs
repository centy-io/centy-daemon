//! Tests for generic storage helpers.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::helpers::{
    copy_dir_contents, copy_item_assets, type_storage_path, update_project_manifest,
};
use crate::manifest;
use mdstore::TypeConfig;
use tokio::fs;

async fn setup_project(temp: &std::path::Path) {
    let centy_path = temp.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = manifest::create_manifest();
    manifest::write_manifest(temp, &manifest).await.unwrap();
}

// ─── type_storage_path ───────────────────────────────────────────────────────

#[test]
fn test_type_storage_path_basic() {
    let path = std::path::Path::new("/some/project");
    let result = type_storage_path(path, "issues");
    assert_eq!(
        result,
        std::path::PathBuf::from("/some/project/.centy/issues")
    );
}

#[test]
fn test_type_storage_path_docs() {
    let path = std::path::Path::new("/my/project");
    let result = type_storage_path(path, "docs");
    assert_eq!(result, std::path::PathBuf::from("/my/project/.centy/docs"));
}

// ─── copy_dir_contents ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_copy_dir_contents_empty_dir() {
    let temp = tempfile::tempdir().unwrap();
    let src = temp.path().join("src");
    let dst = temp.path().join("dst");
    fs::create_dir_all(&src).await.unwrap();
    fs::create_dir_all(&dst).await.unwrap();

    copy_dir_contents(&src, &dst)
        .await
        .expect("Should succeed on empty dir");
    // dst should still exist and be empty
    let mut entries = fs::read_dir(&dst).await.unwrap();
    assert!(entries.next_entry().await.unwrap().is_none());
}

#[tokio::test]
async fn test_copy_dir_contents_copies_files() {
    let temp = tempfile::tempdir().unwrap();
    let src = temp.path().join("src");
    let dst = temp.path().join("dst");
    fs::create_dir_all(&src).await.unwrap();
    fs::create_dir_all(&dst).await.unwrap();

    fs::write(src.join("file.txt"), b"hello world")
        .await
        .unwrap();

    copy_dir_contents(&src, &dst)
        .await
        .expect("Should copy files");

    let content = fs::read_to_string(dst.join("file.txt")).await.unwrap();
    assert_eq!(content, "hello world");
}

#[tokio::test]
async fn test_copy_dir_contents_copies_nested_dirs() {
    let temp = tempfile::tempdir().unwrap();
    let src = temp.path().join("src");
    let dst = temp.path().join("dst");
    let src_sub = src.join("subdir");
    fs::create_dir_all(&src_sub).await.unwrap();
    fs::create_dir_all(&dst).await.unwrap();

    fs::write(src_sub.join("nested.txt"), b"nested content")
        .await
        .unwrap();
    fs::write(src.join("root.txt"), b"root content")
        .await
        .unwrap();

    copy_dir_contents(&src, &dst)
        .await
        .expect("Should copy recursively");

    let root_content = fs::read_to_string(dst.join("root.txt")).await.unwrap();
    assert_eq!(root_content, "root content");

    let nested_content = fs::read_to_string(dst.join("subdir").join("nested.txt"))
        .await
        .unwrap();
    assert_eq!(nested_content, "nested content");
}

#[tokio::test]
async fn test_copy_dir_contents_error_missing_src() {
    let temp = tempfile::tempdir().unwrap();
    let src = temp.path().join("nonexistent");
    let dst = temp.path().join("dst");
    fs::create_dir_all(&dst).await.unwrap();

    let result = copy_dir_contents(&src, &dst).await;
    assert!(result.is_err());
}

// ─── copy_item_assets ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_copy_item_assets_no_assets_feature() {
    let temp = tempfile::tempdir().unwrap();
    let source_project = temp.path().join("source");
    let target_project = temp.path().join("target");

    let config = TypeConfig {
        name: "Note".to_string(),
        identifier: mdstore::IdStrategy::Uuid,
        features: mdstore::TypeFeatures {
            assets: false,
            ..Default::default()
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    let source_dir = source_project.join(".centy").join("notes");
    let result = copy_item_assets(
        &source_project,
        &target_project,
        &source_dir,
        "notes",
        "notes",
        &config,
        "some-id",
        None,
    )
    .await
    .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_copy_item_assets_new_path_exists() {
    let temp = tempfile::tempdir().unwrap();
    let source_project = temp.path().join("source");
    let target_project = temp.path().join("target");

    // Create source assets in new-style location: .centy/assets/issues/item_id/
    let item_id = "my-item-uuid";
    let source_new_assets = source_project
        .join(".centy")
        .join("assets")
        .join("issues")
        .join(item_id);
    fs::create_dir_all(&source_new_assets).await.unwrap();
    fs::write(source_new_assets.join("image.png"), b"png data")
        .await
        .unwrap();

    let config = TypeConfig {
        name: "Issue".to_string(),
        identifier: mdstore::IdStrategy::Uuid,
        features: mdstore::TypeFeatures {
            assets: true,
            ..Default::default()
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    let source_dir = source_project.join(".centy").join("issues");
    let result = copy_item_assets(
        &source_project,
        &target_project,
        &source_dir,
        "issues",
        "issues",
        &config,
        item_id,
        None,
    )
    .await
    .unwrap();

    assert!(result.is_some());

    // Verify the file was copied to target
    let target_assets = target_project
        .join(".centy")
        .join("assets")
        .join("issues")
        .join(item_id);
    assert!(target_assets.join("image.png").exists());
}

#[tokio::test]
async fn test_copy_item_assets_legacy_path_exists() {
    let temp = tempfile::tempdir().unwrap();
    let source_project = temp.path().join("source");
    let target_project = temp.path().join("target");

    // Create source assets in legacy location: .centy/issues/assets/{item_id}/
    let item_id = "my-item-uuid";
    let source_dir = source_project.join(".centy").join("issues");
    let source_legacy_assets = source_dir.join("assets").join(item_id);
    fs::create_dir_all(&source_legacy_assets).await.unwrap();
    fs::write(source_legacy_assets.join("doc.pdf"), b"pdf data")
        .await
        .unwrap();

    let config = TypeConfig {
        name: "Issue".to_string(),
        identifier: mdstore::IdStrategy::Uuid,
        features: mdstore::TypeFeatures {
            assets: true,
            ..Default::default()
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    let result = copy_item_assets(
        &source_project,
        &target_project,
        &source_dir,
        "issues",
        "issues",
        &config,
        item_id,
        None,
    )
    .await
    .unwrap();

    assert!(result.is_some());
}

#[tokio::test]
async fn test_copy_item_assets_no_assets_exists() {
    let temp = tempfile::tempdir().unwrap();
    let source_project = temp.path().join("source");
    let target_project = temp.path().join("target");

    let config = TypeConfig {
        name: "Issue".to_string(),
        identifier: mdstore::IdStrategy::Uuid,
        features: mdstore::TypeFeatures {
            assets: true,
            ..Default::default()
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    let source_dir = source_project.join(".centy").join("issues");
    let result = copy_item_assets(
        &source_project,
        &target_project,
        &source_dir,
        "issues",
        "issues",
        &config,
        "nonexistent-id",
        None,
    )
    .await
    .unwrap();

    // No assets found — returns None
    assert!(result.is_none());
}

#[tokio::test]
async fn test_copy_item_assets_slug_strategy_uses_new_id() {
    let temp = tempfile::tempdir().unwrap();
    let source_project = temp.path().join("source");
    let target_project = temp.path().join("target");

    let item_id = "old-slug";
    let new_id = "new-slug";

    // Create source assets in new-style location
    let source_new_assets = source_project
        .join(".centy")
        .join("assets")
        .join("docs")
        .join(item_id);
    fs::create_dir_all(&source_new_assets).await.unwrap();
    fs::write(source_new_assets.join("file.txt"), b"content")
        .await
        .unwrap();

    let config = TypeConfig {
        name: "Doc".to_string(),
        identifier: mdstore::IdStrategy::Slug,
        features: mdstore::TypeFeatures {
            assets: true,
            ..Default::default()
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    let source_dir = source_project.join(".centy").join("docs");
    let result = copy_item_assets(
        &source_project,
        &target_project,
        &source_dir,
        "docs",
        "docs",
        &config,
        item_id,
        Some(new_id),
    )
    .await
    .unwrap();

    assert!(result.is_some());

    // Verify the file was copied to target with NEW id
    let target_assets = target_project
        .join(".centy")
        .join("assets")
        .join("docs")
        .join(new_id);
    assert!(target_assets.join("file.txt").exists());
}

// ─── update_project_manifest ─────────────────────────────────────────────────

#[tokio::test]
async fn test_update_project_manifest_no_manifest() {
    let temp = tempfile::tempdir().unwrap();
    // Don't write a manifest — function should succeed and do nothing
    let result = update_project_manifest(temp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_project_manifest_with_manifest() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let result = update_project_manifest(temp.path()).await;
    assert!(result.is_ok());

    // Manifest should still exist
    let manifest_path = temp.path().join(".centy").join(".centy-manifest.json");
    assert!(manifest_path.exists());
}
