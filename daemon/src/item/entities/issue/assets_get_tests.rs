//! Additional tests for `get_asset` covering missed branches.
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

// ─── get.rs coverage ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_asset_not_initialized() {
    let temp = tempfile::tempdir().unwrap();
    let result = get_asset(temp.path(), None, "file.png", true).await;
    assert!(matches!(result.unwrap_err(), AssetError::NotInitialized));
}

#[tokio::test]
async fn test_get_asset_missing_issue_id() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    // is_shared=false but issue_id=None => InvalidFilename
    let result = get_asset(temp.path(), None, "file.png", false).await;
    assert!(matches!(
        result.unwrap_err(),
        AssetError::InvalidFilename(_)
    ));
}

#[tokio::test]
async fn test_get_asset_issue_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let result = get_asset(temp.path(), Some("nonexistent-uuid"), "file.png", false).await;
    assert!(matches!(result.unwrap_err(), AssetError::IssueNotFound(_)));
}

#[tokio::test]
async fn test_get_asset_issue_specific_not_found() {
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

    let result = get_asset(temp.path(), Some(&issue.id), "nonexistent.png", false).await;
    assert!(matches!(result.unwrap_err(), AssetError::AssetNotFound(_)));
}

#[tokio::test]
async fn test_get_asset_shared_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let result = get_asset(temp.path(), None, "missing.png", true).await;
    assert!(matches!(result.unwrap_err(), AssetError::AssetNotFound(_)));
}

#[tokio::test]
async fn test_get_asset_issue_specific_legacy_path() {
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

    // Write asset at legacy path
    let legacy_dir = temp
        .path()
        .join(".centy")
        .join("issues")
        .join(&issue.id)
        .join("assets");
    tokio::fs::create_dir_all(&legacy_dir).await.unwrap();
    let test_data = create_test_png();
    tokio::fs::write(legacy_dir.join("legacy.png"), &test_data)
        .await
        .unwrap();

    let (data, info) = get_asset(temp.path(), Some(&issue.id), "legacy.png", false)
        .await
        .unwrap();
    assert_eq!(data, test_data);
    assert_eq!(info.filename, "legacy.png");
    assert!(!info.is_shared);
}

#[tokio::test]
async fn test_get_asset_issue_specific_new_path_preferred() {
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

    // Write asset at new path
    let new_dir = temp
        .path()
        .join(".centy")
        .join("issues")
        .join("assets")
        .join(&issue.id);
    tokio::fs::create_dir_all(&new_dir).await.unwrap();
    let test_data = create_test_png();
    tokio::fs::write(new_dir.join("new.png"), &test_data)
        .await
        .unwrap();

    let (data, info) = get_asset(temp.path(), Some(&issue.id), "new.png", false)
        .await
        .unwrap();
    assert_eq!(data, test_data);
    assert_eq!(info.filename, "new.png");
    assert_eq!(info.mime_type, "image/png");
    assert!(!info.is_shared);
}

#[tokio::test]
async fn test_get_asset_issue_folder_exists_no_asset() {
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

    // Create issue folder but no asset
    let issue_folder = temp.path().join(".centy").join("issues").join(&issue.id);
    tokio::fs::create_dir_all(&issue_folder).await.unwrap();

    let result = get_asset(temp.path(), Some(&issue.id), "missing.png", false).await;
    assert!(matches!(result.unwrap_err(), AssetError::AssetNotFound(_)));
}
