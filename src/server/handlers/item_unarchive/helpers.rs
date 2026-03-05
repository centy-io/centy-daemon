use crate::item::core::error::ItemError;
use crate::server::structured_error::to_error_json;
use mdstore::Item;
/// Determine the destination folder for an unarchive operation.
pub(super) fn resolve_target_folder(
    project_path_str: &str,
    archived_item: &Item,
    requested: &str,
) -> Result<String, String> {
    if !requested.is_empty() {
        return Ok(requested.to_string());
    }
    match archived_item
        .frontmatter
        .custom_fields
        .get("original_item_type")
    {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        _ => {
            let err = ItemError::custom(
                "original_item_type not set on archived item; \
                 provide target_item_type to override",
            );
            Err(to_error_json(project_path_str, &err))
        }
    }
}
