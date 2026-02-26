#![allow(unknown_lints, max_nesting_depth)]
use super::types::AssetError;
use std::path::Path;
use tokio::fs;

pub async fn copy_assets_folder(
    source_assets_path: &Path,
    target_assets_path: &Path,
) -> Result<u32, AssetError> {
    fs::create_dir_all(target_assets_path).await?;
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
            fs::copy(&source_file, &target_file).await?;
            copied_count = copied_count.saturating_add(1);
        }
    }
    Ok(copied_count)
}
