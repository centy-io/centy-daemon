//! Additional tests for `list_shared_assets` covering missed branches.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

async fn setup_project(temp: &std::path::Path) {
    use crate::reconciliation::{execute_reconciliation, ReconciliationDecisions};
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(temp, decisions, true)
        .await
        .expect("Failed to initialize centy project");
}

fn create_test_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

// ─── list_shared.rs coverage ─────────────────────────────────────────────────

#[tokio::test]
async fn test_list_shared_assets_not_initialized() {
    let temp = tempfile::tempdir().unwrap();
    let result = list_shared_assets(temp.path()).await;
    assert!(matches!(result.unwrap_err(), AssetError::NotInitialized));
}

#[tokio::test]
async fn test_list_shared_assets_no_assets_dir() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    // Remove the shared assets dir
    let shared_dir = temp.path().join(".centy").join("assets");
    if shared_dir.exists() {
        tokio::fs::remove_dir_all(&shared_dir).await.unwrap();
    }

    let assets = list_shared_assets(temp.path()).await.unwrap();
    assert!(assets.is_empty());
}

#[tokio::test]
async fn test_list_shared_assets_with_files() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let shared_dir = temp.path().join(".centy").join("assets");
    tokio::fs::create_dir_all(&shared_dir).await.unwrap();
    tokio::fs::write(shared_dir.join("logo.png"), create_test_png())
        .await
        .unwrap();
    tokio::fs::write(shared_dir.join("banner.png"), create_test_png())
        .await
        .unwrap();

    let assets = list_shared_assets(temp.path()).await.unwrap();
    assert_eq!(assets.len(), 2);
    for asset in &assets {
        assert!(asset.is_shared);
        assert!(!asset.hash.is_empty());
        assert!(asset.size > 0);
    }
}

#[tokio::test]
async fn test_list_shared_assets_skips_subdirectories() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let shared_dir = temp.path().join(".centy").join("assets");
    tokio::fs::create_dir_all(&shared_dir).await.unwrap();

    // Create a subdirectory (should be skipped)
    tokio::fs::create_dir_all(shared_dir.join("subdir"))
        .await
        .unwrap();

    // Create a file (should be listed)
    tokio::fs::write(shared_dir.join("icon.png"), create_test_png())
        .await
        .unwrap();

    let assets = list_shared_assets(temp.path()).await.unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].filename, "icon.png");
    assert!(assets[0].is_shared);
}

#[tokio::test]
async fn test_list_shared_assets_empty_dir() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let shared_dir = temp.path().join(".centy").join("assets");
    tokio::fs::create_dir_all(&shared_dir).await.unwrap();
    // Nothing in it

    let assets = list_shared_assets(temp.path()).await.unwrap();
    assert!(assets.is_empty());
}

#[tokio::test]
async fn test_list_shared_assets_mime_type_set() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let shared_dir = temp.path().join(".centy").join("assets");
    tokio::fs::create_dir_all(&shared_dir).await.unwrap();
    tokio::fs::write(shared_dir.join("image.png"), create_test_png())
        .await
        .unwrap();

    let assets = list_shared_assets(temp.path()).await.unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].mime_type, "image/png");
}
