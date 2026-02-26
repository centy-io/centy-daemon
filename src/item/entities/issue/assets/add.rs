#![allow(unknown_lints, max_nesting_depth)]
use super::types::{
    compute_binary_hash, get_mime_type, sanitize_filename, AddAssetResult, AssetError, AssetInfo,
    AssetScope,
};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;

pub async fn add_asset(
    project_path: &Path,
    issue_id: Option<&str>,
    data: Vec<u8>,
    filename: &str,
    scope: AssetScope,
) -> Result<AddAssetResult, AssetError> {
    let sanitized_filename = sanitize_filename(filename)?;
    let mime_type = get_mime_type(&sanitized_filename)
        .ok_or_else(|| AssetError::UnsupportedFileType(sanitized_filename.clone()))?;
    let manifest = read_manifest(project_path)
        .await?
        .ok_or(AssetError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let (assets_dir, manifest_base_path) = match scope {
        AssetScope::IssueSpecific => {
            let id = issue_id.ok_or_else(|| {
                AssetError::InvalidFilename("Issue ID required for issue-specific assets".into())
            })?;
            let issue_file_path = centy_path.join("issues").join(format!("{id}.md"));
            let issue_folder_path = centy_path.join("issues").join(id);
            if !issue_file_path.exists() && !issue_folder_path.exists() {
                return Err(AssetError::IssueNotFound(id.to_string()));
            }
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
    fs::create_dir_all(&assets_dir).await?;
    let asset_path = assets_dir.join(&sanitized_filename);
    if asset_path.exists() {
        return Err(AssetError::AssetAlreadyExists(sanitized_filename));
    }
    let hash = compute_binary_hash(&data);
    let size = data.len() as u64;
    let created_at = now_iso();
    fs::write(&asset_path, &data).await?;
    let mut manifest = manifest;
    update_manifest(&mut manifest);
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
