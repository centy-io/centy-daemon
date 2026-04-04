use super::super::types::{LinkRecord, TargetType};
use super::serialization::{create_link_fields, item_to_link_record, update_link_fields};
use super::validation::{validate_link_ids, validate_link_type};
use crate::utils::get_centy_path;
use mdstore::{CreateOptions, Filters, IdStrategy, TypeConfig, TypeFeatures};
use std::path::Path;

const LINKS_FOLDER: &str = "links";

fn link_type_config() -> TypeConfig {
    TypeConfig {
        name: "link".to_string(),
        identifier: IdStrategy::Uuid,
        features: TypeFeatures::default(),
        statuses: vec![],
        default_status: None,
        priority_levels: None,
        custom_fields: vec![],
    }
}

fn links_dir(project_path: &Path) -> std::path::PathBuf {
    get_centy_path(project_path).join(LINKS_FOLDER)
}

/// Create a new link file in `.centy/links/` and return the full `LinkRecord`.
pub async fn create_link_file(
    project_path: &Path,
    source_id: &str,
    source_type: &TargetType,
    target_id: &str,
    target_type: &TargetType,
    link_type: &str,
) -> Result<LinkRecord, mdstore::StoreError> {
    validate_link_ids(source_id, target_id)?;
    validate_link_type(link_type)?;
    let config = link_type_config();
    let dir = links_dir(project_path);
    let options = CreateOptions {
        title: String::new(),
        body: String::new(),
        id: None,
        status: None,
        priority: None,
        tags: None,
        custom_fields: create_link_fields(source_id, source_type, target_id, target_type, link_type),
        comment: None,
    };
    let item = mdstore::create(&dir, &config, options).await?;
    item_to_link_record(item).ok_or_else(|| {
        mdstore::StoreError::custom("Created link item is missing required custom fields")
    })
}

/// Update an existing link file — currently supports updating `link_type`.
pub async fn update_link_file(
    project_path: &Path,
    link_id: &str,
    link_type: &str,
) -> Result<LinkRecord, mdstore::StoreError> {
    validate_link_type(link_type)?;
    let config = link_type_config();
    let dir = links_dir(project_path);
    let options = mdstore::UpdateOptions {
        title: None,
        body: None,
        status: None,
        priority: None,
        tags: None,
        custom_fields: update_link_fields(link_type),
        comment: None,
    };
    let item = mdstore::update(&dir, &config, link_id, options).await?;
    item_to_link_record(item).ok_or_else(|| {
        mdstore::StoreError::custom("Updated link item is missing required custom fields")
    })
}

/// Hard-delete a link file by UUID.
pub async fn delete_link_file(
    project_path: &Path,
    link_id: &str,
) -> Result<(), mdstore::StoreError> {
    let dir = links_dir(project_path);
    mdstore::delete(&dir, link_id, true).await
}

/// Load all link records from `.centy/links/`, skipping any malformed files.
pub async fn list_all_link_records(
    project_path: &Path,
) -> Result<Vec<LinkRecord>, mdstore::StoreError> {
    let dir = links_dir(project_path);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let items = mdstore::list(&dir, Filters::new()).await?;
    Ok(items.into_iter().filter_map(item_to_link_record).collect())
}
