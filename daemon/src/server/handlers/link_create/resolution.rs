use crate::item::ItemError;
use crate::link::TargetType;
use crate::server::handlers::item_type_resolve::{resolve_item_id, resolve_item_type_config};
use std::path::Path;

/// Resolve the source and target IDs (and their effective types) for a link.
///
/// Either ID may carry an optional `type:` prefix (e.g. `plan:<uuid>`). When the
/// prefix names a known item type, it overrides the default type for that side,
/// which is what makes cross-type linking by full UUID possible. When there is
/// no prefix — or the prefix is not a recognized item type — the default type is
/// used and the raw value is treated as the ID unchanged.
pub(super) async fn resolve_link_ids(
    project_path: &Path,
    source_type: &TargetType,
    target_type: &TargetType,
    source_id: &str,
    target_id: &str,
) -> Result<(String, TargetType, String, TargetType), ItemError> {
    let (resolved_source_id, resolved_source_type) =
        resolve_side(project_path, source_type, source_id).await?;
    let (resolved_target_id, resolved_target_type) =
        resolve_side(project_path, target_type, target_id).await?;
    Ok((
        resolved_source_id,
        resolved_source_type,
        resolved_target_id,
        resolved_target_type,
    ))
}

/// Resolve one side of a link, honoring an optional `type:` prefix on `raw_id`.
///
/// Returns the resolved ID and the type that should be used for it.
async fn resolve_side(
    project_path: &Path,
    default_type: &TargetType,
    raw_id: &str,
) -> Result<(String, TargetType), ItemError> {
    if let Some((prefix, rest)) = raw_id.split_once(':') {
        if !prefix.is_empty() && !rest.is_empty() {
            // Only treat the prefix as a type override when it actually names a
            // configured/built-in item type; otherwise it is part of the ID.
            if let Ok((folder, config)) = resolve_item_type_config(project_path, prefix).await {
                let id = resolve_item_id(project_path, &folder, &config, rest).await?;
                return Ok((id, TargetType::new(prefix.to_lowercase())));
            }
        }
    }
    let id = resolve_one_id(project_path, default_type.as_str(), raw_id).await?;
    Ok((id, default_type.clone()))
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
