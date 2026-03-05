use super::crud_helpers::{entity_exists, get_entity_path};
use super::crud_types::{
    CreateLinkOptions, CreateLinkResult, DeleteLinkOptions, DeleteLinkResult, LinkError,
};
use super::{
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
pub async fn delete_link(
    project_path: &Path,
    options: DeleteLinkOptions,
    custom_types: &[CustomLinkTypeDefinition],
) -> Result<DeleteLinkResult, LinkError> {
    let source_path = get_entity_path(project_path, &options.source_id, &options.source_type);
    let target_path = get_entity_path(project_path, &options.target_id, &options.target_type);
    let mut source_links = read_links(&source_path).await?;
    let mut target_links = read_links(&target_path).await?;
    let mut deleted_count = 0u32;
    if let Some(link_type) = &options.link_type {
        if source_links.remove_link(&options.target_id, Some(link_type)) {
            deleted_count = deleted_count.saturating_add(1);
        }
        if let Some(inverse_type) = get_inverse_link_type(link_type, custom_types) {
            if target_links.remove_link(&options.source_id, Some(&inverse_type)) {
                deleted_count = deleted_count.saturating_add(1);
            }
        }
    } else {
        let link_types: Vec<String> = source_links
            .links
            .iter()
            .filter(|l| l.target_id == options.target_id)
            .map(|l| l.link_type.clone())
            .collect();
        if source_links.remove_link(&options.target_id, None) {
            deleted_count = deleted_count.saturating_add(link_types.len().try_into().unwrap_or(u32::MAX));
        }
        for link_type in &link_types {
            if let Some(inverse_type) = get_inverse_link_type(link_type, custom_types) {
                if target_links.remove_link(&options.source_id, Some(&inverse_type)) {
                    deleted_count = deleted_count.saturating_add(1);
                }
            }
        }
    }
    if deleted_count == 0 {
        return Err(LinkError::LinkNotFound);
    }
    write_links(&source_path, &source_links).await?;
    write_links(&target_path, &target_links).await?;
    Ok(DeleteLinkResult { deleted_count })
}
