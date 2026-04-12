use super::super::crud_types::{LinkError, UpdateLinkOptions};
use super::super::link_types::is_valid_link_type;
use super::super::storage::update_link_file;
use super::super::types::{CustomLinkTypeDefinition, LinkRecord};
use std::path::Path;

/// Update a link's `link_type` by its UUID.
pub async fn update_link(
    project_path: &Path,
    options: UpdateLinkOptions,
    custom_types: &[CustomLinkTypeDefinition],
) -> Result<LinkRecord, LinkError> {
    if !is_valid_link_type(&options.link_type, custom_types) {
        return Err(LinkError::InvalidLinkType(options.link_type.clone()));
    }
    match update_link_file(project_path, &options.link_id, &options.link_type).await {
        Ok(record) => Ok(record),
        Err(mdstore::StoreError::NotFound(_)) => Err(LinkError::LinkNotFound),
        Err(e) => Err(LinkError::StoreError(e)),
    }
}
