//! Move operation for generic items.
use super::helpers::{copy_item_assets, type_storage_path, update_project_manifest};
use crate::item::core::error::ItemError;
use crate::manifest;
use mdstore::TypeConfig;
use std::path::Path;
use tokio::fs;
/// Move an item from one project to another.
pub async fn generic_move(
    source_project_path: &Path,
    target_project_path: &Path,
    source_folder: &str,
    target_folder: &str,
    source_config: &TypeConfig,
    target_config: &TypeConfig,
    item_id: &str,
    new_id: Option<&str>,
) -> Result<mdstore::MoveResult, ItemError> {
    manifest::read_manifest(source_project_path)
        .await?
        .ok_or(ItemError::NotInitialized)?;
    manifest::read_manifest(target_project_path)
        .await?
        .ok_or(ItemError::TargetNotInitialized)?;
    let source_dir = type_storage_path(source_project_path, source_folder);
    let target_dir = type_storage_path(target_project_path, target_folder);
    let copied_assets = copy_item_assets(
        source_project_path,
        target_project_path,
        &source_dir,
        source_folder,
        target_folder,
        source_config,
        item_id,
        new_id,
    )
    .await?;
    let result = mdstore::move_item(
        &source_dir,
        &target_dir,
        source_config,
        target_config,
        item_id,
        new_id,
    )
    .await?;
    if let Some(src_assets) = copied_assets {
        if src_assets.exists() {
            fs::remove_dir_all(&src_assets).await?;
        }
    }
    if source_config.features.assets {
        let source_assets_legacy = source_dir.join("assets").join(item_id);
        if source_assets_legacy.exists() {
            fs::remove_dir_all(&source_assets_legacy).await?;
        }
    }
    update_project_manifest(source_project_path).await?;
    update_project_manifest(target_project_path).await?;
    Ok(result)
}
