#![allow(unknown_lints, max_nesting_depth)]
use super::types::{compute_binary_hash, get_mime_type, sanitize_filename, AssetError, AssetInfo};
use crate::manifest::read_manifest;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;

pub async fn get_asset(
    project_path: &Path,
    issue_id: Option<&str>,
    filename: &str,
    is_shared: bool,
) -> Result<(Vec<u8>, AssetInfo), AssetError> {
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
        let issue_file_path = centy_path.join("issues").join(format!("{id}.md"));
        let issue_folder_path = centy_path.join("issues").join(id);
        if !issue_file_path.exists() && !issue_folder_path.exists() {
            return Err(AssetError::IssueNotFound(id.to_string()));
        }
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
    if !asset_path.exists() {
        return Err(AssetError::AssetNotFound(sanitized_filename));
    }
    let data = fs::read(&asset_path).await?;
    let size = data.len() as u64;
    let hash = compute_binary_hash(&data);
    let mime_type = get_mime_type(&sanitized_filename)
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
