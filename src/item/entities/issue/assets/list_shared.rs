#![allow(unknown_lints, max_nesting_depth)]
use super::types::{compute_binary_hash, get_mime_type, AssetError, AssetInfo};
use crate::manifest::read_manifest;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;

pub async fn list_shared_assets(project_path: &Path) -> Result<Vec<AssetInfo>, AssetError> {
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
