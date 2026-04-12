use crate::item::ItemError;
use crate::link::TargetType;
use crate::server::handlers::item_type_resolve::{resolve_item_id, resolve_item_type_config};
use std::path::Path;

pub(super) async fn resolve_link_ids(
    project_path: &Path,
    source_type: &TargetType,
    target_type: &TargetType,
    source_id: &str,
    target_id: &str,
) -> Result<(String, String), ItemError> {
    let resolved_source = resolve_one_id(project_path, source_type.as_str(), source_id).await?;
    let resolved_target = resolve_one_id(project_path, target_type.as_str(), target_id).await?;
    Ok((resolved_source, resolved_target))
}

async fn resolve_one_id(
    project_path: &Path,
    item_type: &str,
    raw_id: &str,
) -> Result<String, ItemError> {
    match resolve_item_type_config(project_path, item_type).await {
        Ok((folder, config)) => resolve_item_id(project_path, &folder, &config, raw_id).await,
        Err(_) => Ok(raw_id.to_string()),
    }
}
