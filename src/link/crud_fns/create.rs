use super::super::crud_helpers::{entity_exists, get_entity_path};
use super::super::crud_types::{CreateLinkOptions, CreateLinkResult, LinkError};
use super::super::{
    get_inverse_link_type, is_valid_link_type, read_links, write_links, CustomLinkTypeDefinition,
    Link,
};
use std::path::Path;
pub async fn create_link(
    project_path: &Path,
    options: CreateLinkOptions,
    custom_types: &[CustomLinkTypeDefinition],
) -> Result<CreateLinkResult, LinkError> {
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
    let inverse_type = get_inverse_link_type(&options.link_type, custom_types)
        .ok_or_else(|| LinkError::InvalidLinkType(options.link_type.clone()))?;
    let source_path = get_entity_path(project_path, &options.source_id, &options.source_type);
    let target_path = get_entity_path(project_path, &options.target_id, &options.target_type);
    let mut source_links = read_links(&source_path).await?;
    let mut target_links = read_links(&target_path).await?;
    if source_links.has_link(&options.target_id, &options.link_type) {
        return Err(LinkError::LinkAlreadyExists);
    }
    let forward_link = Link::new(
        options.target_id.clone(),
        options.target_type,
        options.link_type.clone(),
    );
    let inverse_link = Link::new(options.source_id.clone(), options.source_type, inverse_type);
    source_links.add_link(forward_link.clone());
    target_links.add_link(inverse_link.clone());
    write_links(&source_path, &source_links).await?;
    write_links(&target_path, &target_links).await?;
    Ok(CreateLinkResult {
        created_link: forward_link,
        inverse_link,
    })
}
