use super::super::types::{LinkRecord, TargetType};
use crate::utils::get_centy_path;
use mdstore::{CreateOptions, Filters, IdStrategy, TypeConfig, TypeFeatures};
use serde_json::json;
use std::collections::HashMap;
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

fn item_to_link_record(item: mdstore::Item) -> Option<LinkRecord> {
    let cf = &item.frontmatter.custom_fields;
    let source_id = cf.get("sourceId")?.as_str()?.to_string();
    let source_type = TargetType::new(cf.get("sourceType")?.as_str()?);
    let target_id = cf.get("targetId")?.as_str()?.to_string();
    let target_type = TargetType::new(cf.get("targetType")?.as_str()?);
    let link_type = cf.get("linkType")?.as_str()?.to_string();
    Some(LinkRecord {
        id: item.id,
        source_id,
        source_type,
        target_id,
        target_type,
        link_type,
        created_at: item.frontmatter.created_at,
        updated_at: item.frontmatter.updated_at,
    })
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
    let config = link_type_config();
    let dir = links_dir(project_path);
    let mut fields = HashMap::new();
    fields.insert("sourceId".to_string(), json!(source_id));
    fields.insert("sourceType".to_string(), json!(source_type.as_str()));
    fields.insert("targetId".to_string(), json!(target_id));
    fields.insert("targetType".to_string(), json!(target_type.as_str()));
    fields.insert("linkType".to_string(), json!(link_type));
    let options = CreateOptions {
        title: String::new(),
        body: String::new(),
        id: None,
        status: None,
        priority: None,
        tags: None,
        custom_fields: fields,
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
    let config = link_type_config();
    let dir = links_dir(project_path);
    let mut fields = std::collections::HashMap::new();
    fields.insert("linkType".to_string(), json!(link_type));
    let options = mdstore::UpdateOptions {
        title: None,
        body: None,
        status: None,
        priority: None,
        tags: None,
        custom_fields: fields,
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
