//! Additional tests for `list_assets` covering missed branches.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::item::entities::issue::{create_issue, CreateIssueOptions};

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

// ─── list.rs coverage ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_assets_not_initialized() {
    let temp = tempfile::tempdir().unwrap();
    let result = list_assets(temp.path(), "some-uuid", false).await;
    assert!(matches!(result.unwrap_err(), AssetError::NotInitialized));
}

#[tokio::test]
async fn test_list_assets_issue_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let result = list_assets(temp.path(), "nonexistent-uuid", false).await;
    assert!(matches!(result.unwrap_err(), AssetError::IssueNotFound(_)));
}

#[tokio::test]
async fn test_list_assets_with_old_assets_path() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Write asset at OLD (legacy) path
    let old_assets_dir = temp
        .path()
        .join(".centy")
        .join("issues")
        .join(&issue.id)
        .join("assets");
    tokio::fs::create_dir_all(&old_assets_dir).await.unwrap();
    tokio::fs::write(old_assets_dir.join("old.png"), create_test_png())
        .await
        .unwrap();

    let assets = list_assets(temp.path(), &issue.id, false).await.unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].filename, "old.png");
    assert!(!assets[0].is_shared);
}

#[tokio::test]
async fn test_list_assets_new_path_only() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Write asset at NEW path
    let new_assets_dir = temp
        .path()
        .join(".centy")
        .join("issues")
        .join("assets")
        .join(&issue.id);
    tokio::fs::create_dir_all(&new_assets_dir).await.unwrap();
    tokio::fs::write(new_assets_dir.join("new.png"), create_test_png())
        .await
        .unwrap();

    let assets = list_assets(temp.path(), &issue.id, false).await.unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].filename, "new.png");
    assert!(!assets[0].is_shared);
}

#[tokio::test]
async fn test_list_assets_with_shared_included() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Write a shared asset
    let shared_dir = temp.path().join(".centy").join("assets");
    tokio::fs::create_dir_all(&shared_dir).await.unwrap();
    tokio::fs::write(shared_dir.join("shared.png"), create_test_png())
        .await
        .unwrap();

    let assets = list_assets(temp.path(), &issue.id, true).await.unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].filename, "shared.png");
    assert!(assets[0].is_shared);
}

#[tokio::test]
async fn test_list_assets_include_shared_no_shared_dir() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Remove the shared assets dir if it was created
    let shared_dir = temp.path().join(".centy").join("assets");
    if shared_dir.exists() {
        tokio::fs::remove_dir_all(&shared_dir).await.unwrap();
    }

    // Asking for shared but no shared dir — should return empty
    let assets = list_assets(temp.path(), &issue.id, true).await.unwrap();
    assert!(assets.is_empty());
}

#[tokio::test]
async fn test_scan_assets_directory_skips_subdirectories() {
    let temp = tempfile::tempdir().unwrap();
    let assets_dir = temp.path().join("assets");
    tokio::fs::create_dir_all(&assets_dir).await.unwrap();

    // Create a subdirectory (should be skipped)
    tokio::fs::create_dir_all(assets_dir.join("subdir"))
        .await
        .unwrap();

    // Create a file (should be listed)
    tokio::fs::write(assets_dir.join("file.png"), create_test_png())
        .await
        .unwrap();

    let mut result = Vec::new();
    scan_assets_directory(&assets_dir, &mut result, false)
        .await
        .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].filename, "file.png");
}

#[tokio::test]
async fn test_list_assets_using_issue_folder_path_exists() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Create issue folder path (so the folder-exists branch is taken)
    let issue_folder = temp.path().join(".centy").join("issues").join(&issue.id);
    tokio::fs::create_dir_all(&issue_folder).await.unwrap();

    // No assets dir — should return empty
    let assets = list_assets(temp.path(), &issue.id, false).await.unwrap();
    assert!(assets.is_empty());
}
