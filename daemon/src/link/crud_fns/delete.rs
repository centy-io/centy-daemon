use super::super::crud_types::{DeleteLinkOptions, DeleteLinkResult, LinkError};
use super::super::storage::delete_link_file;
use std::path::Path;

pub async fn delete_link(
    project_path: &Path,
    options: DeleteLinkOptions,
) -> Result<DeleteLinkResult, LinkError> {
    match delete_link_file(project_path, &options.link_id).await {
        Ok(()) => Ok(DeleteLinkResult { deleted_count: 1 }),
        Err(mdstore::StoreError::NotFound(_)) => Err(LinkError::LinkNotFound),
        Err(e) => Err(LinkError::StoreError(e)),
    }
}
