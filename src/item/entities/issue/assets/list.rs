#![allow(unknown_lints, max_nesting_depth)]
use super::types::{AssetError, AssetInfo, compute_binary_hash, get_mime_type};
use crate::manifest::read_manifest;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;

pub async fn list_assets(
    project_path: &Path,
    issue_id: &str,
    include_shared: bool,
) -> Result<Vec<AssetInfo>, AssetError> {
    read_manifest(project_path)
        .await?
        .ok_or(AssetError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let issue_file_path = centy_path.join("issues").join(format!("{issue_id}.md"));
    let issue_folder_path = centy_path.join("issues").join(issue_id);
    if !issue_file_path.exists() && !issue_folder_path.exists() {
        return Err(AssetError::IssueNotFound(issue_id.to_string()));
    }
    let mut assets = Vec::new();
    let new_assets_path = centy_path.join("issues").join("assets").join(issue_id);
    let old_assets_path = issue_folder_path.join("assets");
    if new_assets_path.exists() {
        scan_assets_directory(&new_assets_path, &mut assets, false).await?;
    }
    if old_assets_path.exists() {
        scan_assets_directory(&old_assets_path, &mut assets, false).await?;
    }
    if include_shared {
        let shared_assets_path = centy_path.join("assets");
        if shared_assets_path.exists() {
            scan_assets_directory(&shared_assets_path, &mut assets, true).await?;
        }
    }
    Ok(assets)
}

pub async fn scan_assets_directory(
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
                    hash, size, mime_type, is_shared, created_at,
                });
            }
        }
    }
    Ok(())
}
