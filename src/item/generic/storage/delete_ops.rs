use super::helpers::{type_storage_path, update_project_manifest};
use crate::item::core::error::ItemError;
use crate::utils::get_centy_path;
use mdstore::TypeConfig;
use std::path::Path;
use tokio::fs;
/// Delete item assets directory if it exists.
async fn delete_item_assets(project_path: &Path, folder: &str, id: &str) -> Result<(), ItemError> {
    let assets_path = get_centy_path(project_path)
        .join("assets")
        .join(folder)
        .join(id);
    if assets_path.exists() {
        fs::remove_dir_all(&assets_path).await?;
    }
    Ok(())
}
/// Delete an item (hard or soft delete).
pub async fn generic_delete(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    id: &str,
    force: bool,
) -> Result<(), ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    mdstore::delete(&type_dir, id, force).await?;
    if force && config.features.assets {
        delete_item_assets(project_path, folder, id).await?;
    }
    update_project_manifest(project_path).await?;
    Ok(())
}
/// Soft-delete an item by setting the `deleted_at` timestamp.
pub async fn generic_soft_delete(
    project_path: &Path,
    folder: &str,
    id: &str,
) -> Result<(), ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    mdstore::soft_delete(&type_dir, id).await?;
    update_project_manifest(project_path).await?;
    Ok(())
}
/// Restore a soft-deleted item by clearing the `deleted_at` timestamp.
pub async fn generic_restore(project_path: &Path, folder: &str, id: &str) -> Result<(), ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    mdstore::restore(&type_dir, id).await?;
    update_project_manifest(project_path).await?;
    Ok(())
}
