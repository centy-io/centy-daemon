//! Basic CRUD operations for generic items.
use super::helpers::{type_storage_path, update_project_manifest};
use crate::item::core::error::ItemError;
use mdstore::{CreateOptions, Filters, TypeConfig, UpdateOptions};
use std::path::Path;
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
