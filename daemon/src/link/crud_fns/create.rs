use super::super::crud_helpers::entity_exists;
use super::super::crud_types::{CreateLinkOptions, LinkError};
use super::super::link_types::is_valid_link_type;
use super::super::storage::{create_link_file, list_all_link_records};
use super::super::types::{CustomLinkTypeDefinition, LinkRecord};
use std::path::Path;

pub async fn create_link(
    project_path: &Path,
    options: CreateLinkOptions,
    custom_types: &[CustomLinkTypeDefinition],
) -> Result<LinkRecord, LinkError> {
    if !is_valid_link_type(&options.link_type, custom_types) {
        return Err(LinkError::InvalidLinkType(options.link_type));
    }
    if options.source_id == options.target_id && options.source_type == options.target_type {
        return Err(LinkError::SelfLink);
    }
    if !entity_exists(project_path, &options.source_id, &options.source_type) {
        return Err(LinkError::SourceNotFound(
            options.source_id.clone(),
            options.source_type.clone(),
        ));
    }
    if !entity_exists(project_path, &options.target_id, &options.target_type) {
        return Err(LinkError::TargetNotFound(
            options.target_id.clone(),
            options.target_type.clone(),
        ));
    }
    // Check for duplicate link.
    let existing = list_all_link_records(project_path).await?;
    let already_exists = existing.iter().any(|r| {
        r.source_id == options.source_id
            && r.target_id == options.target_id
            && r.link_type == options.link_type
    });
    if already_exists {
        return Err(LinkError::LinkAlreadyExists);
    }
    Ok(create_link_file(
        project_path,
        &options.source_id,
        &options.source_type,
        &options.target_id,
        &options.target_type,
        &options.link_type,
    )
    .await?)
}
