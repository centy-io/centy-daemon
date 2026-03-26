//! Additional tests for `delete_asset` covering missed branches.
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

// ─── delete.rs coverage ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_asset_not_initialized() {
    let temp = tempfile::tempdir().unwrap();
    // Do NOT initialize project
    let result = delete_asset(temp.path(), Some("some-id"), "file.png", false).await;
    assert!(matches!(result.unwrap_err(), AssetError::NotInitialized));
}

#[tokio::test]
async fn test_delete_asset_missing_issue_id_for_issue_specific() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    // issue_id = None but is_shared = false => InvalidFilename
    let result = delete_asset(temp.path(), None, "file.png", false).await;
    assert!(matches!(
        result.unwrap_err(),
        AssetError::InvalidFilename(_)
    ));
}

#[tokio::test]
async fn test_delete_asset_issue_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let result = delete_asset(temp.path(), Some("nonexistent-uuid"), "file.png", false).await;
    assert!(matches!(result.unwrap_err(), AssetError::IssueNotFound(_)));
}

#[tokio::test]
async fn test_delete_asset_shared_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let result = delete_asset(temp.path(), None, "missing.png", true).await;
    assert!(matches!(result.unwrap_err(), AssetError::AssetNotFound(_)));
}

#[tokio::test]
async fn test_delete_asset_shared_success() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    // Write a PNG to the shared assets folder directly
    let shared_dir = temp.path().join(".centy").join("assets");
    tokio::fs::create_dir_all(&shared_dir).await.unwrap();
    tokio::fs::write(shared_dir.join("logo.png"), create_test_png())
        .await
        .unwrap();

    let result = delete_asset(temp.path(), None, "logo.png", true)
        .await
        .unwrap();
    assert_eq!(result.filename, "logo.png");
    assert!(result.was_shared);
    assert!(!shared_dir.join("logo.png").exists());
}

#[tokio::test]
async fn test_delete_asset_issue_specific_uses_new_path() {
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

    // Manually write asset at new path
    let new_asset_dir = temp
        .path()
        .join(".centy")
        .join("issues")
        .join("assets")
        .join(&issue.id);
    tokio::fs::create_dir_all(&new_asset_dir).await.unwrap();
    tokio::fs::write(new_asset_dir.join("test.png"), create_test_png())
        .await
        .unwrap();

    let result = delete_asset(temp.path(), Some(&issue.id), "test.png", false)
        .await
        .unwrap();
    assert_eq!(result.filename, "test.png");
    assert!(!result.was_shared);
    assert!(!new_asset_dir.join("test.png").exists());
}

#[tokio::test]
async fn test_delete_asset_issue_specific_uses_legacy_path() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Legacy Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Manually write asset at OLD (legacy) path: .centy/issues/{id}/assets/
    let legacy_asset_dir = temp
        .path()
        .join(".centy")
        .join("issues")
        .join(&issue.id)
        .join("assets");
    tokio::fs::create_dir_all(&legacy_asset_dir).await.unwrap();
    tokio::fs::write(legacy_asset_dir.join("legacy.png"), create_test_png())
        .await
        .unwrap();

    let result = delete_asset(temp.path(), Some(&issue.id), "legacy.png", false)
        .await
        .unwrap();
    assert_eq!(result.filename, "legacy.png");
}

#[tokio::test]
async fn test_delete_asset_issue_folder_exists_but_no_asset() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Issue No Asset".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Make the issue_folder_path exist (so IssueNotFound is not returned)
    let issue_folder = temp.path().join(".centy").join("issues").join(&issue.id);
    tokio::fs::create_dir_all(&issue_folder).await.unwrap();

    let result = delete_asset(temp.path(), Some(&issue.id), "missing.png", false).await;
    assert!(matches!(result.unwrap_err(), AssetError::AssetNotFound(_)));
}
