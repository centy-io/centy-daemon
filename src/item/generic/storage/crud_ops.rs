//! Basic CRUD operations for generic items.
use super::helpers::{type_storage_path, update_project_manifest};
use crate::item::core::error::ItemError;
use crate::utils::get_centy_path;
use mdstore::{CreateOptions, Filters, TypeConfig, UpdateOptions};
use std::path::Path;
use tokio::fs;
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
/// Get a single generic item by display number.
pub async fn generic_get_by_display_number(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    display_number: u32,
) -> Result<mdstore::Item, ItemError> {
    if !config.features.display_number {
        return Err(ItemError::FeatureNotEnabled(
            "display_number is not enabled for this item type".to_string(),
        ));
    }
    let type_dir = type_storage_path(project_path, folder);
    let items = mdstore::list(&type_dir, Filters::new().include_deleted()).await?;
    for item in items {
        if item.frontmatter.display_number == Some(display_number) {
            return Ok(item);
        }
    }
    Err(ItemError::NotFound(format!(
        "display_number {display_number}"
    )))
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
/// Delete an item (hard or soft delete).
#[allow(unknown_lints, max_nesting_depth)]
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
        let assets_path = get_centy_path(project_path)
            .join("assets")
            .join(folder)
            .join(id);
        if assets_path.exists() {
            fs::remove_dir_all(&assets_path).await?;
        }
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
