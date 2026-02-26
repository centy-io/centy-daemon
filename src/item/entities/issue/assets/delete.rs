#![allow(unknown_lints, max_nesting_depth)]
use super::types::{AssetError, DeleteAssetResult, sanitize_filename};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;

pub async fn delete_asset(
    project_path: &Path,
    issue_id: Option<&str>,
    filename: &str,
    is_shared: bool,
) -> Result<DeleteAssetResult, AssetError> {
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
        if new_path.exists() { new_path } else { old_path }
    };
    if !asset_path.exists() {
        return Err(AssetError::AssetNotFound(sanitized_filename));
    }
    fs::remove_file(&asset_path).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    Ok(DeleteAssetResult { filename: sanitized_filename, was_shared: is_shared })
}
