//! Shared helpers for generic storage operations.
use crate::item::core::error::ItemError;
use crate::manifest;
use crate::utils::get_centy_path;
use mdstore::TypeConfig;
use std::path::{Path, PathBuf};
use tokio::fs;
/// Get the storage directory for a given item type.
pub fn type_storage_path(project_path: &Path, folder: &str) -> std::path::PathBuf {
    get_centy_path(project_path).join(folder)
}
/// Recursively copy the contents of one directory to another.
pub async fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), ItemError> {
    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type().await?.is_dir() {
            fs::create_dir_all(&dst_path).await?;
            Box::pin(copy_dir_contents(&src_path, &dst_path)).await?;
        } else {
            fs::copy(&src_path, &dst_path).await?;
        }
    }
    Ok(())
}
/// Copy item assets from source to target project, returning the source asset path if found.
pub async fn copy_item_assets(
    source_project_path: &Path,
    target_project_path: &Path,
    source_dir: &Path,
    source_folder: &str,
    target_folder: &str,
    source_config: &TypeConfig,
    item_id: &str,
    new_id: Option<&str>,
) -> Result<Option<PathBuf>, ItemError> {
    if !source_config.features.assets {
        return Ok(None);
    }
    let source_assets_new = get_centy_path(source_project_path)
        .join("assets")
        .join(source_folder)
        .join(item_id);
    let source_assets_legacy = source_dir.join("assets").join(item_id);
    let source_assets = if source_assets_new.exists() {
        Some(source_assets_new)
    } else if source_assets_legacy.exists() {
        Some(source_assets_legacy)
    } else {
        None
    };
    if let Some(src_assets) = &source_assets {
        let target_id = if source_config.identifier == mdstore::IdStrategy::Slug {
            new_id.unwrap_or(item_id)
        } else {
            item_id
        };
        let target_assets = get_centy_path(target_project_path)
            .join("assets")
            .join(target_folder)
            .join(target_id);
        fs::create_dir_all(&target_assets).await?;
        copy_dir_contents(src_assets, &target_assets).await?;
    }
    Ok(source_assets)
}
/// Helper to update the project manifest timestamp.
pub async fn update_project_manifest(project_path: &Path) -> Result<(), ItemError> {
    if let Some(mut m) = manifest::read_manifest(project_path).await? {
        manifest::update_manifest(&mut m);
        manifest::write_manifest(project_path, &m).await?;
    }
    Ok(())
}
