use super::super::crud_types::LinkError;
use super::super::storage::delete_links_for_entity;
use std::path::Path;

/// Hard-delete all links that reference the given entity (as source or target).
///
/// Called automatically as part of hard-deleting an item so no orphan link
/// records are ever left behind.
pub async fn cascade_delete_entity_links(
    project_path: &Path,
    entity_id: &str,
) -> Result<u32, LinkError> {
    Ok(delete_links_for_entity(project_path, entity_id).await?)
}
