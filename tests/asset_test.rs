mod common;

use centy_daemon::issue::{
    add_asset, create_issue, delete_asset, get_asset, list_assets, list_shared_assets,
    AssetError, AssetScope, CreateIssueOptions,
};
use common::{create_test_dir, init_centy_project};

/// Create a simple PNG image (1x1 pixel transparent)
fn create_test_png() -> Vec<u8> {
    // Minimal valid PNG (1x1 transparent pixel)
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, // IEND chunk
        0x42, 0x60, 0x82,
    ]
}

/// Create a simple JPEG image (minimal valid)
fn create_test_jpeg() -> Vec<u8> {
    // Minimal valid JPEG (1x1 pixel)
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, // Start of image + APP0
        0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
        0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, // Quantization table
        0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08,
        0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C,
        0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
        0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D,
        0x1A, 0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20,
        0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
        0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27,
        0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34,
        0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, // Start of frame
        0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, // Huffman table
        0x00, 0x1F, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04,
        0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0xFF,
        0xC4, 0x00, 0xB5, 0x10, 0x00, 0x02, 0x01, 0x03,
        0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04,
        0x00, 0x00, 0x01, 0x7D, 0xFF, 0xDA, 0x00, 0x08, // Start of scan
        0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, 0x7F, 0xFF,
        0xD9, // End of image
    ]
}

// ============ Add Asset Tests ============

#[tokio::test]
async fn test_add_asset_to_issue_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create an issue first
    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    // Add an asset to the issue
    let png_data = create_test_png();
    let result = add_asset(
        project_path,
        Some(&issue.id),
        png_data.clone(),
        "screenshot.png",
        AssetScope::IssueSpecific,
    )
    .await
    .expect("Should add asset");

    assert_eq!(result.asset.filename, "screenshot.png");
    assert_eq!(result.asset.mime_type, "image/png");
    assert_eq!(result.asset.size, png_data.len() as u64);
    assert!(!result.asset.is_shared);
    assert!(!result.asset.hash.is_empty());

    // Verify file exists
    let asset_path = project_path
        .join(".centy/issues")
        .join(&issue.id)
        .join("assets")
        .join("screenshot.png");
    assert!(asset_path.exists());
}

#[tokio::test]
async fn test_add_shared_asset_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Add a shared asset (no issue required)
    let png_data = create_test_png();
    let result = add_asset(
        project_path,
        None,
        png_data.clone(),
        "logo.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add shared asset");

    assert_eq!(result.asset.filename, "logo.png");
    assert!(result.asset.is_shared);

    // Verify file exists in shared assets
    let asset_path = project_path.join(".centy/assets/logo.png");
    assert!(asset_path.exists());
}

#[tokio::test]
async fn test_add_asset_requires_init() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Don't initialize - try to add asset
    let result = add_asset(
        project_path,
        None,
        create_test_png(),
        "test.png",
        AssetScope::Shared,
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AssetError::NotInitialized));
}

#[tokio::test]
async fn test_add_asset_issue_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Try to add asset to non-existent issue
    let result = add_asset(
        project_path,
        Some("nonexistent-uuid"),
        create_test_png(),
        "test.png",
        AssetScope::IssueSpecific,
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AssetError::IssueNotFound(_)));
}

#[tokio::test]
async fn test_add_asset_already_exists() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Add first asset
    add_asset(
        project_path,
        None,
        create_test_png(),
        "logo.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add first asset");

    // Try to add same filename
    let result = add_asset(
        project_path,
        None,
        create_test_png(),
        "logo.png",
        AssetScope::Shared,
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AssetError::AssetAlreadyExists(_)));
}

#[tokio::test]
async fn test_add_asset_invalid_filename() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Empty filename
    let result = add_asset(
        project_path,
        None,
        create_test_png(),
        "",
        AssetScope::Shared,
    )
    .await;
    assert!(matches!(result.unwrap_err(), AssetError::InvalidFilename(_)));

    // Path traversal
    let result = add_asset(
        project_path,
        None,
        create_test_png(),
        "../evil.png",
        AssetScope::Shared,
    )
    .await;
    assert!(matches!(result.unwrap_err(), AssetError::InvalidFilename(_)));

    // Path with slashes
    let result = add_asset(
        project_path,
        None,
        create_test_png(),
        "path/to/file.png",
        AssetScope::Shared,
    )
    .await;
    assert!(matches!(result.unwrap_err(), AssetError::InvalidFilename(_)));

    // Hidden file
    let result = add_asset(
        project_path,
        None,
        create_test_png(),
        ".hidden.png",
        AssetScope::Shared,
    )
    .await;
    assert!(matches!(result.unwrap_err(), AssetError::InvalidFilename(_)));
}

#[tokio::test]
async fn test_add_asset_unsupported_file_type() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Unsupported file type
    let result = add_asset(
        project_path,
        None,
        vec![0, 1, 2, 3],
        "document.txt",
        AssetScope::Shared,
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AssetError::UnsupportedFileType(_)));
}

#[tokio::test]
async fn test_add_asset_various_file_types() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Test PNG
    let result = add_asset(
        project_path,
        None,
        create_test_png(),
        "image.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add PNG");
    assert_eq!(result.asset.mime_type, "image/png");

    // Test JPEG
    let result = add_asset(
        project_path,
        None,
        create_test_jpeg(),
        "image.jpg",
        AssetScope::Shared,
    )
    .await
    .expect("Should add JPEG");
    assert_eq!(result.asset.mime_type, "image/jpeg");

    // Test JPEG with .jpeg extension
    let result = add_asset(
        project_path,
        None,
        create_test_jpeg(),
        "another.jpeg",
        AssetScope::Shared,
    )
    .await
    .expect("Should add JPEG");
    assert_eq!(result.asset.mime_type, "image/jpeg");

    // Test case insensitive extension
    let result = add_asset(
        project_path,
        None,
        create_test_png(),
        "UPPERCASE.PNG",
        AssetScope::Shared,
    )
    .await
    .expect("Should add PNG with uppercase extension");
    assert_eq!(result.asset.mime_type, "image/png");
}

// ============ List Assets Tests ============

#[tokio::test]
async fn test_list_assets_empty() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create an issue
    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    // List assets - should be empty
    let assets = list_assets(project_path, &issue.id, false)
        .await
        .expect("Should list assets");

    assert!(assets.is_empty());
}

#[tokio::test]
async fn test_list_assets_returns_issue_specific() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create an issue
    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    // Add two assets
    add_asset(
        project_path,
        Some(&issue.id),
        create_test_png(),
        "first.png",
        AssetScope::IssueSpecific,
    )
    .await
    .expect("Should add first");

    add_asset(
        project_path,
        Some(&issue.id),
        create_test_jpeg(),
        "second.jpg",
        AssetScope::IssueSpecific,
    )
    .await
    .expect("Should add second");

    // List assets
    let assets = list_assets(project_path, &issue.id, false)
        .await
        .expect("Should list");

    assert_eq!(assets.len(), 2);
    let filenames: Vec<_> = assets.iter().map(|a| a.filename.as_str()).collect();
    assert!(filenames.contains(&"first.png"));
    assert!(filenames.contains(&"second.jpg"));
}

#[tokio::test]
async fn test_list_assets_includes_shared_when_requested() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create an issue and add issue-specific asset
    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    add_asset(
        project_path,
        Some(&issue.id),
        create_test_png(),
        "issue-asset.png",
        AssetScope::IssueSpecific,
    )
    .await
    .expect("Should add");

    // Add shared asset
    add_asset(
        project_path,
        None,
        create_test_png(),
        "shared-asset.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add shared");

    // List without shared
    let assets = list_assets(project_path, &issue.id, false)
        .await
        .expect("Should list");
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].filename, "issue-asset.png");

    // List with shared
    let assets = list_assets(project_path, &issue.id, true)
        .await
        .expect("Should list");
    assert_eq!(assets.len(), 2);
    let filenames: Vec<_> = assets.iter().map(|a| a.filename.as_str()).collect();
    assert!(filenames.contains(&"issue-asset.png"));
    assert!(filenames.contains(&"shared-asset.png"));
}

#[tokio::test]
async fn test_list_shared_assets() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Add multiple shared assets
    add_asset(
        project_path,
        None,
        create_test_png(),
        "logo.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add");

    add_asset(
        project_path,
        None,
        create_test_jpeg(),
        "banner.jpg",
        AssetScope::Shared,
    )
    .await
    .expect("Should add");

    // List shared assets
    let assets = list_shared_assets(project_path)
        .await
        .expect("Should list");

    assert_eq!(assets.len(), 2);
    for asset in &assets {
        assert!(asset.is_shared);
    }
}

// ============ Get Asset Tests ============

#[tokio::test]
async fn test_get_asset_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue and add asset
    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let original_data = create_test_png();
    add_asset(
        project_path,
        Some(&issue.id),
        original_data.clone(),
        "test.png",
        AssetScope::IssueSpecific,
    )
    .await
    .expect("Should add");

    // Get the asset
    let (data, info) = get_asset(project_path, Some(&issue.id), "test.png", false)
        .await
        .expect("Should get asset");

    assert_eq!(data, original_data);
    assert_eq!(info.filename, "test.png");
    assert_eq!(info.size, original_data.len() as u64);
}

#[tokio::test]
async fn test_get_shared_asset_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let original_data = create_test_png();
    add_asset(
        project_path,
        None,
        original_data.clone(),
        "shared.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add");

    // Get the shared asset
    let (data, info) = get_asset(project_path, None, "shared.png", true)
        .await
        .expect("Should get");

    assert_eq!(data, original_data);
    assert!(info.is_shared);
}

#[tokio::test]
async fn test_get_asset_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let result = get_asset(project_path, Some(&issue.id), "nonexistent.png", false).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AssetError::AssetNotFound(_)));
}

// ============ Delete Asset Tests ============

#[tokio::test]
async fn test_delete_asset_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    // Add asset
    add_asset(
        project_path,
        Some(&issue.id),
        create_test_png(),
        "to-delete.png",
        AssetScope::IssueSpecific,
    )
    .await
    .expect("Should add");

    // Verify exists
    let asset_path = project_path
        .join(".centy/issues")
        .join(&issue.id)
        .join("assets")
        .join("to-delete.png");
    assert!(asset_path.exists());

    // Delete
    let result = delete_asset(project_path, Some(&issue.id), "to-delete.png", false)
        .await
        .expect("Should delete");

    assert_eq!(result.filename, "to-delete.png");
    assert!(!result.was_shared);

    // Verify gone
    assert!(!asset_path.exists());

    // Verify not in list
    let assets = list_assets(project_path, &issue.id, false)
        .await
        .expect("Should list");
    assert!(assets.is_empty());
}

#[tokio::test]
async fn test_delete_shared_asset_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Add shared asset
    add_asset(
        project_path,
        None,
        create_test_png(),
        "shared.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add");

    // Delete
    let result = delete_asset(project_path, None, "shared.png", true)
        .await
        .expect("Should delete");

    assert_eq!(result.filename, "shared.png");
    assert!(result.was_shared);

    // Verify gone
    let asset_path = project_path.join(".centy/assets/shared.png");
    assert!(!asset_path.exists());
}

#[tokio::test]
async fn test_delete_asset_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let result = delete_asset(project_path, Some(&issue.id), "nonexistent.png", false).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AssetError::AssetNotFound(_)));
}

// ============ Manifest Integration Tests ============

#[tokio::test]
async fn test_add_asset_updates_manifest() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Add shared asset
    add_asset(
        project_path,
        None,
        create_test_png(),
        "tracked.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add");

    // Check manifest
    let manifest = centy_daemon::manifest::read_manifest(project_path)
        .await
        .unwrap()
        .unwrap();

    let asset_entry = manifest
        .managed_files
        .iter()
        .find(|f| f.path == "assets/tracked.png");

    assert!(asset_entry.is_some(), "Asset should be in manifest");
    let entry = asset_entry.unwrap();
    assert!(!entry.hash.is_empty(), "Asset should have a hash");
}

#[tokio::test]
async fn test_delete_asset_removes_from_manifest() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Add and then delete asset
    add_asset(
        project_path,
        None,
        create_test_png(),
        "temp.png",
        AssetScope::Shared,
    )
    .await
    .expect("Should add");

    delete_asset(project_path, None, "temp.png", true)
        .await
        .expect("Should delete");

    // Check manifest
    let manifest = centy_daemon::manifest::read_manifest(project_path)
        .await
        .unwrap()
        .unwrap();

    let asset_entry = manifest
        .managed_files
        .iter()
        .find(|f| f.path == "assets/temp.png");

    assert!(asset_entry.is_none(), "Asset should not be in manifest");
}

// ============ Edge Cases ============

#[tokio::test]
async fn test_asset_scope_default_is_issue_specific() {
    let scope: AssetScope = Default::default();
    assert_eq!(scope, AssetScope::IssueSpecific);
}

#[tokio::test]
async fn test_add_asset_requires_issue_id_for_issue_specific() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Try to add issue-specific asset without issue ID
    let result = add_asset(
        project_path,
        None, // No issue ID
        create_test_png(),
        "test.png",
        AssetScope::IssueSpecific, // But scope is issue-specific
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AssetError::InvalidFilename(_)));
}

#[tokio::test]
async fn test_assets_isolated_between_issues() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create two issues
    let issue1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create");

    let issue2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create");

    // Add asset to issue 1
    add_asset(
        project_path,
        Some(&issue1.id),
        create_test_png(),
        "unique.png",
        AssetScope::IssueSpecific,
    )
    .await
    .expect("Should add");

    // Issue 1 should have the asset
    let assets1 = list_assets(project_path, &issue1.id, false)
        .await
        .expect("Should list");
    assert_eq!(assets1.len(), 1);

    // Issue 2 should NOT have the asset
    let assets2 = list_assets(project_path, &issue2.id, false)
        .await
        .expect("Should list");
    assert_eq!(assets2.len(), 0);
}
