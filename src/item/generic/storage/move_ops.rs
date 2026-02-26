//! Move and duplicate operations for generic items.
#![allow(unknown_lints, max_nesting_depth)]
use super::helpers::{copy_dir_contents, type_storage_path, update_project_manifest};
use crate::item::core::error::ItemError;
use crate::manifest;
use crate::utils::get_centy_path;
use mdstore::TypeConfig;
use std::path::Path;
use tokio::fs;
use super::super::types::DuplicateGenericItemOptions;
/// Duplicate an item to the same or different project.
pub async fn generic_duplicate(
    folder: &str, config: &TypeConfig, options: DuplicateGenericItemOptions,
) -> Result<mdstore::DuplicateResult, ItemError> {
    let source_dir = type_storage_path(&options.source_project_path, folder);
    let target_dir = type_storage_path(&options.target_project_path, folder);
    let mdstore_options = mdstore::DuplicateOptions {
        source_dir: source_dir.clone(),
        target_dir: target_dir.clone(),
        item_id: options.item_id.clone(),
        new_id: options.new_id,
        new_title: options.new_title,
    };
    let result = mdstore::duplicate(config, mdstore_options).await?;
    if config.features.assets {
        let source_assets = source_dir.join("assets").join(&options.item_id);
        let target_assets = target_dir.join("assets").join(&result.item.id);
        if source_assets.exists() {
            fs::create_dir_all(&target_assets).await?;
            copy_dir_contents(&source_assets, &target_assets).await?;
        }
    }
    update_project_manifest(&options.target_project_path).await?;
    Ok(result)
}
/// Move an item from one project to another.
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn generic_move(
    source_project_path: &Path, target_project_path: &Path,
    source_folder: &str, target_folder: &str,
    source_config: &TypeConfig, target_config: &TypeConfig,
    item_id: &str, new_id: Option<&str>,
) -> Result<mdstore::MoveResult, ItemError> {
    manifest::read_manifest(source_project_path).await?.ok_or(ItemError::NotInitialized)?;
    manifest::read_manifest(target_project_path).await?.ok_or(ItemError::TargetNotInitialized)?;
    let source_dir = type_storage_path(source_project_path, source_folder);
    let target_dir = type_storage_path(target_project_path, target_folder);
    let copied_assets = if source_config.features.assets {
        let source_assets_new = get_centy_path(source_project_path).join("assets").join(source_folder).join(item_id);
        let source_assets_legacy = source_dir.join("assets").join(item_id);
        let source_assets = if source_assets_new.exists() { Some(source_assets_new) }
            else if source_assets_legacy.exists() { Some(source_assets_legacy) } else { None };
        if let Some(ref src_assets) = source_assets {
            let target_id = if source_config.identifier == mdstore::IdStrategy::Slug { new_id.unwrap_or(item_id) } else { item_id };
            let target_assets = get_centy_path(target_project_path).join("assets").join(target_folder).join(target_id);
            fs::create_dir_all(&target_assets).await?;
            copy_dir_contents(src_assets, &target_assets).await?;
        }
        source_assets
    } else { None };
    let result = mdstore::move_item(&source_dir, &target_dir, source_config, target_config, item_id, new_id).await?;
    if let Some(src_assets) = copied_assets {
        if src_assets.exists() { fs::remove_dir_all(&src_assets).await?; }
    }
    if source_config.features.assets {
        let source_assets_legacy = source_dir.join("assets").join(item_id);
        if source_assets_legacy.exists() { fs::remove_dir_all(&source_assets_legacy).await?; }
    }
    update_project_manifest(source_project_path).await?;
    update_project_manifest(target_project_path).await?;
    Ok(result)
}
/// Rename a slug-based item within the same project folder.
pub async fn generic_rename_slug(
    project_path: &Path, folder: &str, _config: &TypeConfig, item_id: &str, new_id: &str,
) -> Result<mdstore::MoveResult, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    let source_file = type_dir.join(format!("{item_id}.md"));
    let target_file = type_dir.join(format!("{new_id}.md"));
    if !source_file.exists() { return Err(ItemError::NotFound(item_id.to_string())); }
    if target_file.exists() { return Err(ItemError::Custom(format!("item with id '{new_id}' already exists"))); }
    let mut item = mdstore::get(&type_dir, item_id).await?;
    item.id = new_id.to_string();
    tokio::fs::rename(&source_file, &target_file).await?;
    update_project_manifest(project_path).await?;
    Ok(mdstore::MoveResult { item, old_id: item_id.to_string() })
}
