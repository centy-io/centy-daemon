//! Asset management for issues
//!
//! Provides functionality to add, list, retrieve, and delete assets (images, videos, etc.)
//! attached to issues. Assets can be either issue-specific or shared across all issues.

use crate::manifest::{read_manifest, update_manifest_timestamp, write_manifest};
use crate::utils::{get_centy_path, now_iso};
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

/// Asset scope: where the asset is stored
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssetScope {
    /// Asset is specific to an issue (stored in `.centy/issues/{id}/assets/`)
    #[default]
    IssueSpecific,
    /// Asset is shared across all issues (stored in `.centy/assets/`)
    Shared,
}

/// Information about an asset
#[derive(Debug, Clone)]
pub struct AssetInfo {
    /// Filename of the asset
    pub filename: String,
    /// SHA-256 hash of the file contents
    pub hash: String,
    /// File size in bytes
    pub size: u64,
    /// MIME type (e.g., "image/png", "video/mp4")
    pub mime_type: String,
    /// Whether this is a shared asset
    pub is_shared: bool,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
}

/// Errors that can occur during asset operations
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Issue not found: {0}")]
    IssueNotFound(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Asset already exists: {0}")]
    AssetAlreadyExists(String),

    #[error("Invalid filename: {0}")]
    InvalidFilename(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
}

/// Result of adding an asset
#[derive(Debug, Clone)]
pub struct AddAssetResult {
    /// Information about the added asset
    pub asset: AssetInfo,
    /// Full path to the asset file (relative to project root)
    pub path: String,
}

/// Result of deleting an asset
#[derive(Debug, Clone)]
pub struct DeleteAssetResult {
    /// Filename of the deleted asset
    pub filename: String,
    /// Whether it was a shared asset
    pub was_shared: bool,
}

/// Supported image MIME types
const IMAGE_MIME_TYPES: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("svg", "image/svg+xml"),
    ("ico", "image/x-icon"),
    ("bmp", "image/bmp"),
];

/// Supported video MIME types
const VIDEO_MIME_TYPES: &[(&str, &str)] = &[
    ("mp4", "video/mp4"),
    ("webm", "video/webm"),
    ("mov", "video/quicktime"),
    ("avi", "video/x-msvideo"),
    ("mkv", "video/x-matroska"),
];

/// Compute SHA-256 hash of binary data
fn compute_binary_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Get MIME type from filename extension
fn get_mime_type(filename: &str) -> Option<String> {
    let extension = filename.rsplit('.').next()?.to_lowercase();

    // Check image types
    for (ext, mime) in IMAGE_MIME_TYPES {
        if extension == *ext {
            return Some((*mime).to_string());
        }
    }

    // Check video types
    for (ext, mime) in VIDEO_MIME_TYPES {
        if extension == *ext {
            return Some((*mime).to_string());
        }
    }

    None
}

/// Validate and sanitize a filename
fn sanitize_filename(filename: &str) -> Result<String, AssetError> {
    // Reject empty filenames
    if filename.is_empty() {
        return Err(AssetError::InvalidFilename(
            "Filename cannot be empty".to_string(),
        ));
    }

    // Reject path traversal attempts
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AssetError::InvalidFilename(
            "Filename cannot contain path separators or '..'".to_string(),
        ));
    }

    // Reject hidden files
    if filename.starts_with('.') {
        return Err(AssetError::InvalidFilename(
            "Filename cannot start with '.'".to_string(),
        ));
    }

    // Reject filenames that are too long
    if filename.len() > 255 {
        return Err(AssetError::InvalidFilename(
            "Filename too long (max 255 characters)".to_string(),
        ));
    }

    Ok(filename.to_string())
}

/// Add an asset to an issue or as a shared asset
///
/// # Arguments
/// * `project_path` - Path to the project root
/// * `issue_id` - Issue ID (UUID). Required for issue-specific assets.
/// * `data` - Binary content of the asset
/// * `filename` - Name to save the asset as
/// * `scope` - Whether to store as issue-specific or shared
///
/// # Returns
/// Information about the added asset
pub async fn add_asset(
    project_path: &Path,
    issue_id: Option<&str>,
    data: Vec<u8>,
    filename: &str,
    scope: AssetScope,
) -> Result<AddAssetResult, AssetError> {
    // Validate filename
    let sanitized_filename = sanitize_filename(filename)?;

    // Validate file type
    let mime_type = get_mime_type(&sanitized_filename)
        .ok_or_else(|| AssetError::UnsupportedFileType(sanitized_filename.clone()))?;

    // Check if centy is initialized
    let manifest = read_manifest(project_path)
        .await?
        .ok_or(AssetError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);

    // Determine storage path based on scope
    let (assets_dir, manifest_base_path) = match scope {
        AssetScope::IssueSpecific => {
            let id = issue_id.ok_or_else(|| {
                AssetError::InvalidFilename("Issue ID required for issue-specific assets".into())
            })?;

            // Verify issue exists (check both new format .md file and old format folder)
            let issue_file_path = centy_path.join("issues").join(format!("{id}.md"));
            let issue_folder_path = centy_path.join("issues").join(id);
            if !issue_file_path.exists() && !issue_folder_path.exists() {
                return Err(AssetError::IssueNotFound(id.to_string()));
            }

            // New asset path: .centy/issues/assets/{id}/
            let assets_dir = centy_path.join("issues").join("assets").join(id);
            let manifest_base = format!("issues/assets/{id}/");
            (assets_dir, manifest_base)
        }
        AssetScope::Shared => {
            let assets_dir = centy_path.join("assets");
            let manifest_base = "assets/".to_string();
            (assets_dir, manifest_base)
        }
    };

    // Ensure assets directory exists
    fs::create_dir_all(&assets_dir).await?;

    // Check if asset already exists
    let asset_path = assets_dir.join(&sanitized_filename);
    if asset_path.exists() {
        return Err(AssetError::AssetAlreadyExists(sanitized_filename));
    }

    // Compute hash
    let hash = compute_binary_hash(&data);
    let size = data.len() as u64;
    let created_at = now_iso();

    // Write the file
    fs::write(&asset_path, &data).await?;

    // Update manifest timestamp
    let mut manifest = manifest;
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let asset_info = AssetInfo {
        filename: sanitized_filename.clone(),
        hash,
        size,
        mime_type,
        is_shared: scope == AssetScope::Shared,
        created_at,
    };

    Ok(AddAssetResult {
        asset: asset_info,
        path: format!(".centy/{manifest_base_path}{sanitized_filename}"),
    })
}

/// List all assets for an issue
///
/// # Arguments
/// * `project_path` - Path to the project root
/// * `issue_id` - Issue ID (UUID)
/// * `include_shared` - Whether to include shared assets in the result
///
/// # Returns
/// List of assets for the issue
pub async fn list_assets(
    project_path: &Path,
    issue_id: &str,
    include_shared: bool,
) -> Result<Vec<AssetInfo>, AssetError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(AssetError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);

    // Verify issue exists (check both new format .md file and old format folder)
    let issue_file_path = centy_path.join("issues").join(format!("{issue_id}.md"));
    let issue_folder_path = centy_path.join("issues").join(issue_id);
    if !issue_file_path.exists() && !issue_folder_path.exists() {
        return Err(AssetError::IssueNotFound(issue_id.to_string()));
    }

    let mut assets = Vec::new();

    // Get issue-specific assets by scanning assets directories (both old and new locations)
    // New location: .centy/issues/assets/{id}/
    let new_assets_path = centy_path.join("issues").join("assets").join(issue_id);
    // Old location: .centy/issues/{id}/assets/
    let old_assets_path = issue_folder_path.join("assets");

    // Scan new location first
    if new_assets_path.exists() {
        scan_assets_directory(&new_assets_path, &mut assets, false).await?;
    }
    // Also scan old location if it exists
    if old_assets_path.exists() {
        scan_assets_directory(&old_assets_path, &mut assets, false).await?;
    }

    // Get shared assets if requested
    if include_shared {
        let shared_assets_path = centy_path.join("assets");
        if shared_assets_path.exists() {
            scan_assets_directory(&shared_assets_path, &mut assets, true).await?;
        }
    }

    Ok(assets)
}

/// Helper to scan a directory for assets
async fn scan_assets_directory(
    dir_path: &Path,
    assets: &mut Vec<AssetInfo>,
    is_shared: bool,
) -> Result<(), AssetError> {
    let mut entries = fs::read_dir(dir_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            if let Some(filename) = entry.file_name().to_str() {
                let asset_path = entry.path();
                let data = fs::read(&asset_path).await?;
                let hash = compute_binary_hash(&data);
                let size = data.len() as u64;
                let mime_type = get_mime_type(filename)
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                let metadata = fs::metadata(&asset_path).await?;
                let created_at = metadata
                    .created()
                    .map(|t| {
                        chrono::DateTime::<chrono::Utc>::from(t)
                            .format("%Y-%m-%dT%H:%M:%S%.6f+00:00")
                            .to_string()
                    })
                    .unwrap_or_else(|_| now_iso());

                assets.push(AssetInfo {
                    filename: filename.to_string(),
                    hash,
                    size,
                    mime_type,
                    is_shared,
                    created_at,
                });
            }
        }
    }
    Ok(())
}

/// Get a specific asset's data
///
/// # Arguments
/// * `project_path` - Path to the project root
/// * `issue_id` - Issue ID (UUID) - can be None for shared assets
/// * `filename` - Asset filename
/// * `is_shared` - Whether to look for a shared asset
///
/// # Returns
/// Tuple of (binary data, asset info)
pub async fn get_asset(
    project_path: &Path,
    issue_id: Option<&str>,
    filename: &str,
    is_shared: bool,
) -> Result<(Vec<u8>, AssetInfo), AssetError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(AssetError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let sanitized_filename = sanitize_filename(filename)?;

    let asset_path = if is_shared {
        centy_path.join("assets").join(&sanitized_filename)
    } else {
        let id = issue_id.ok_or_else(|| {
            AssetError::InvalidFilename("Issue ID required for issue-specific assets".into())
        })?;

        // Verify issue exists (check both new format .md file and old format folder)
        let issue_file_path = centy_path.join("issues").join(format!("{id}.md"));
        let issue_folder_path = centy_path.join("issues").join(id);
        if !issue_file_path.exists() && !issue_folder_path.exists() {
            return Err(AssetError::IssueNotFound(id.to_string()));
        }

        // Try new location first, then old location
        let new_path = centy_path
            .join("issues")
            .join("assets")
            .join(id)
            .join(&sanitized_filename);
        let old_path = issue_folder_path.join("assets").join(&sanitized_filename);

        if new_path.exists() {
            new_path
        } else {
            old_path
        }
    };

    // Check if file exists
    if !asset_path.exists() {
        return Err(AssetError::AssetNotFound(sanitized_filename));
    }

    // Read the file
    let data = fs::read(&asset_path).await?;
    let size = data.len() as u64;
    let hash = compute_binary_hash(&data);
    let mime_type = get_mime_type(&sanitized_filename)
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Get created_at from file metadata
    let metadata = fs::metadata(&asset_path).await?;
    let created_at = metadata
        .created()
        .map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
                .format("%Y-%m-%dT%H:%M:%S%.6f+00:00")
                .to_string()
        })
        .unwrap_or_else(|_| now_iso());

    let asset_info = AssetInfo {
        filename: sanitized_filename,
        hash,
        size,
        mime_type,
        is_shared,
        created_at,
    };

    Ok((data, asset_info))
}

/// Delete an asset
///
/// # Arguments
/// * `project_path` - Path to the project root
/// * `issue_id` - Issue ID (UUID) - can be None for shared assets
/// * `filename` - Asset filename
/// * `is_shared` - Whether to delete a shared asset
///
/// # Returns
/// Result of the deletion
pub async fn delete_asset(
    project_path: &Path,
    issue_id: Option<&str>,
    filename: &str,
    is_shared: bool,
) -> Result<DeleteAssetResult, AssetError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(AssetError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let sanitized_filename = sanitize_filename(filename)?;

    let asset_path = if is_shared {
        centy_path.join("assets").join(&sanitized_filename)
    } else {
        let id = issue_id.ok_or_else(|| {
            AssetError::InvalidFilename("Issue ID required for issue-specific assets".into())
        })?;

        // Verify issue exists (check both new format .md file and old format folder)
        let issue_file_path = centy_path.join("issues").join(format!("{id}.md"));
        let issue_folder_path = centy_path.join("issues").join(id);
        if !issue_file_path.exists() && !issue_folder_path.exists() {
            return Err(AssetError::IssueNotFound(id.to_string()));
        }

        // Try new location first, then old location
        let new_path = centy_path
            .join("issues")
            .join("assets")
            .join(id)
            .join(&sanitized_filename);
        let old_path = issue_folder_path.join("assets").join(&sanitized_filename);

        if new_path.exists() {
            new_path
        } else {
            old_path
        }
    };

    // Check if file exists
    if !asset_path.exists() {
        return Err(AssetError::AssetNotFound(sanitized_filename));
    }

    // Delete the file
    fs::remove_file(&asset_path).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(DeleteAssetResult {
        filename: sanitized_filename,
        was_shared: is_shared,
    })
}

/// List all shared assets
///
/// # Arguments
/// * `project_path` - Path to the project root
///
/// # Returns
/// List of shared assets
pub async fn list_shared_assets(project_path: &Path) -> Result<Vec<AssetInfo>, AssetError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(AssetError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let mut assets = Vec::new();

    let shared_assets_path = centy_path.join("assets");
    if shared_assets_path.exists() {
        let mut entries = fs::read_dir(&shared_assets_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(filename) = entry.file_name().to_str() {
                    let asset_path = entry.path();
                    let data = fs::read(&asset_path).await?;
                    let hash = compute_binary_hash(&data);
                    let size = data.len() as u64;
                    let mime_type = get_mime_type(filename)
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    let metadata = fs::metadata(&asset_path).await?;
                    let created_at = metadata
                        .created()
                        .map(|t| {
                            chrono::DateTime::<chrono::Utc>::from(t)
                                .format("%Y-%m-%dT%H:%M:%S%.6f+00:00")
                                .to_string()
                        })
                        .unwrap_or_else(|_| now_iso());

                    assets.push(AssetInfo {
                        filename: filename.to_string(),
                        hash,
                        size,
                        mime_type,
                        is_shared: true,
                        created_at,
                    });
                }
            }
        }
    }

    Ok(assets)
}

/// Copy all assets from one issue folder to another
///
/// # Arguments
/// * `source_assets_path` - Path to source issue's assets folder
/// * `target_assets_path` - Path to target issue's assets folder
///
/// # Returns
/// Number of files copied
pub async fn copy_assets_folder(
    source_assets_path: &Path,
    target_assets_path: &Path,
) -> Result<u32, AssetError> {
    // Ensure target directory exists
    fs::create_dir_all(target_assets_path).await?;

    // If source doesn't exist, nothing to copy
    if !source_assets_path.exists() {
        return Ok(0);
    }

    let mut copied_count = 0u32;

    let mut entries = fs::read_dir(source_assets_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            let source_file = entry.path();
            let filename = entry.file_name();
            let target_file = target_assets_path.join(&filename);

            // Copy the file
            fs::copy(&source_file, &target_file).await?;
            copied_count += 1;
        }
    }

    Ok(copied_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_valid() {
        assert!(sanitize_filename("test.png").is_ok());
        assert!(sanitize_filename("my-image_01.jpg").is_ok());
        assert!(sanitize_filename("screenshot 2024.png").is_ok());
    }

    #[test]
    fn test_sanitize_filename_invalid() {
        assert!(sanitize_filename("").is_err());
        assert!(sanitize_filename("../test.png").is_err());
        assert!(sanitize_filename("path/to/file.png").is_err());
        assert!(sanitize_filename(".hidden").is_err());
        assert!(sanitize_filename(&"a".repeat(300)).is_err());
    }

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type("test.png"), Some("image/png".to_string()));
        assert_eq!(get_mime_type("test.PNG"), Some("image/png".to_string()));
        assert_eq!(get_mime_type("test.jpg"), Some("image/jpeg".to_string()));
        assert_eq!(get_mime_type("test.jpeg"), Some("image/jpeg".to_string()));
        assert_eq!(get_mime_type("test.mp4"), Some("video/mp4".to_string()));
        assert_eq!(get_mime_type("test.webm"), Some("video/webm".to_string()));
        assert_eq!(get_mime_type("test.txt"), None);
        assert_eq!(get_mime_type("test"), None);
    }

    #[test]
    fn test_compute_binary_hash() {
        let data = b"hello world";
        let hash = compute_binary_hash(data);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_asset_scope_default() {
        let scope: AssetScope = Default::default();
        assert_eq!(scope, AssetScope::IssueSpecific);
    }
}
