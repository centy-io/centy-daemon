//! Basic CRUD operations for generic items.
use super::helpers::{type_storage_path, update_project_manifest};
use crate::item::core::error::ItemError;
use crate::utils::get_centy_path;
use mdstore::{CreateOptions, Filters, TypeConfig, UpdateOptions};
use std::path::Path;
use tokio::fs;
use tracing::warn;
/// Create a new generic item.
pub async fn generic_create(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    options: CreateOptions,
) -> Result<mdstore::Item, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    let item = mdstore::create(&type_dir, config, options).await?;
    update_project_manifest(project_path).await?;
    Ok(item)
}
/// Get a single generic item by ID.
pub async fn generic_get(
    project_path: &Path,
    folder: &str,
    id: &str,
) -> Result<mdstore::Item, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    Ok(mdstore::get(&type_dir, id).await?)
}
/// List generic items with optional filters.
pub async fn generic_list(
    project_path: &Path,
    folder: &str,
    filters: Filters,
) -> Result<Vec<mdstore::Item>, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    Ok(mdstore::list(&type_dir, filters).await?)
}
/// Update an existing generic item.
pub async fn generic_update(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    id: &str,
    options: UpdateOptions,
) -> Result<mdstore::Item, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    let item = mdstore::update(&type_dir, config, id, options).await?;
    update_project_manifest(project_path).await?;
    Ok(item)
}
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
    if force {
        // Cascade-delete all links referencing this entity before removing the
        // item itself so no orphan link records are ever left behind.
        if let Err(e) = crate::link::cascade_delete_entity_links(project_path, id).await {
            warn!(id = %id, folder = %folder, error = %e, "Failed to cascade-delete entity links");
        }
    }
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
