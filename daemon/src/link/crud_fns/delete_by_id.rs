use super::super::crud_types::{DeleteLinkResult, LinkError};
use super::super::storage::delete_link_file;
use std::path::Path;

/// Delete a link by its UUID (`Link.id` in proto).
pub async fn delete_link_by_id(
    project_path: &Path,
    link_id: &str,
) -> Result<DeleteLinkResult, LinkError> {
    match delete_link_file(project_path, link_id).await {
        Ok(()) => Ok(DeleteLinkResult { deleted_count: 1 }),
        Err(mdstore::StoreError::NotFound(_)) => Err(LinkError::LinkNotFound),
        Err(e) => Err(LinkError::StoreError(e)),
    }
}
