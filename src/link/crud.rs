use super::{get_inverse_link_type, is_valid_link_type, read_links, write_links, CustomLinkTypeDefinition, Link, LinksFile, TargetType, BUILTIN_LINK_TYPES};
use crate::utils::get_centy_path;
use std::path::Path;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum LinkError {
    #[error("IO error: {0}")] IoError(#[from] std::io::Error),
    #[error("Invalid link type: {0}")] InvalidLinkType(String),
    #[error("Source entity not found: {0} ({1})")] SourceNotFound(String, TargetType),
    #[error("Target entity not found: {0} ({1})")] TargetNotFound(String, TargetType),
    #[error("Link already exists")] LinkAlreadyExists,
    #[error("Link not found")] LinkNotFound,
    #[error("Cannot link entity to itself")] SelfLink,
}
#[derive(Debug, Clone)]
pub struct CreateLinkOptions { pub source_id: String, pub source_type: TargetType, pub target_id: String, pub target_type: TargetType, pub link_type: String }
#[derive(Debug)]
pub struct CreateLinkResult { pub created_link: Link, pub inverse_link: Link }
#[derive(Debug, Clone)]
pub struct DeleteLinkOptions { pub source_id: String, pub source_type: TargetType, pub target_id: String, pub target_type: TargetType, pub link_type: Option<String> }
#[derive(Debug)]
pub struct DeleteLinkResult { pub deleted_count: u32 }
#[derive(Debug, Clone)]
pub struct LinkTypeInfo { pub name: String, pub inverse: String, pub description: Option<String>, pub is_builtin: bool }
fn get_entity_path(project_path: &Path, entity_id: &str, entity_type: TargetType) -> std::path::PathBuf {
    get_centy_path(project_path).join(entity_type.folder_name()).join(entity_id)
}
fn entity_exists(project_path: &Path, entity_id: &str, entity_type: TargetType) -> bool {
    let base = get_centy_path(project_path).join(entity_type.folder_name());
    base.join(format!("{entity_id}.md")).exists() || base.join(entity_id).exists()
}
pub async fn create_link(project_path: &Path, options: CreateLinkOptions, custom_types: &[CustomLinkTypeDefinition]) -> Result<CreateLinkResult, LinkError> {
    if !is_valid_link_type(&options.link_type, custom_types) { return Err(LinkError::InvalidLinkType(options.link_type)); }
    if options.source_id == options.target_id && options.source_type == options.target_type { return Err(LinkError::SelfLink); }
    if !entity_exists(project_path, &options.source_id, options.source_type) { return Err(LinkError::SourceNotFound(options.source_id.clone(), options.source_type)); }
    if !entity_exists(project_path, &options.target_id, options.target_type) { return Err(LinkError::TargetNotFound(options.target_id.clone(), options.target_type)); }
    let inverse_type = get_inverse_link_type(&options.link_type, custom_types).ok_or_else(|| LinkError::InvalidLinkType(options.link_type.clone()))?;
    let source_path = get_entity_path(project_path, &options.source_id, options.source_type);
    let target_path = get_entity_path(project_path, &options.target_id, options.target_type);
    let mut source_links = read_links(&source_path).await?;
    let mut target_links = read_links(&target_path).await?;
    if source_links.has_link(&options.target_id, &options.link_type) { return Err(LinkError::LinkAlreadyExists); }
    let forward_link = Link::new(options.target_id.clone(), options.target_type, options.link_type.clone());
    let inverse_link = Link::new(options.source_id.clone(), options.source_type, inverse_type);
    source_links.add_link(forward_link.clone());
    target_links.add_link(inverse_link.clone());
    write_links(&source_path, &source_links).await?;
    write_links(&target_path, &target_links).await?;
    Ok(CreateLinkResult { created_link: forward_link, inverse_link })
}
pub async fn delete_link(project_path: &Path, options: DeleteLinkOptions, custom_types: &[CustomLinkTypeDefinition]) -> Result<DeleteLinkResult, LinkError> {
    let source_path = get_entity_path(project_path, &options.source_id, options.source_type);
    let target_path = get_entity_path(project_path, &options.target_id, options.target_type);
    let mut source_links = read_links(&source_path).await?;
    let mut target_links = read_links(&target_path).await?;
    let mut deleted_count = 0u32;
    if let Some(link_type) = &options.link_type {
        if source_links.remove_link(&options.target_id, Some(link_type)) { deleted_count = deleted_count.saturating_add(1); }
        if let Some(inv) = get_inverse_link_type(link_type, custom_types) {
            if target_links.remove_link(&options.source_id, Some(&inv)) { deleted_count = deleted_count.saturating_add(1); }
        }
    } else {
        let link_types: Vec<String> = source_links.links.iter().filter(|l| l.target_id == options.target_id).map(|l| l.link_type.clone()).collect();
        if source_links.remove_link(&options.target_id, None) { deleted_count = deleted_count.saturating_add(link_types.len() as u32); }
        for link_type in &link_types {
            if let Some(inv) = get_inverse_link_type(link_type, custom_types) {
                if target_links.remove_link(&options.source_id, Some(&inv)) { deleted_count = deleted_count.saturating_add(1); }
            }
        }
    }
    if deleted_count == 0 { return Err(LinkError::LinkNotFound); }
    write_links(&source_path, &source_links).await?;
    write_links(&target_path, &target_links).await?;
    Ok(DeleteLinkResult { deleted_count })
}
pub async fn list_links(project_path: &Path, entity_id: &str, entity_type: TargetType) -> Result<LinksFile, LinkError> {
    if !entity_exists(project_path, entity_id, entity_type) { return Err(LinkError::SourceNotFound(entity_id.to_string(), entity_type)); }
    Ok(read_links(&get_entity_path(project_path, entity_id, entity_type)).await?)
}
pub fn get_available_link_types(custom_types: &[CustomLinkTypeDefinition]) -> Vec<LinkTypeInfo> {
    let mut types = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (name, inverse) in BUILTIN_LINK_TYPES {
        if !seen.contains(name) && !seen.contains(inverse) {
            types.push(LinkTypeInfo { name: (*name).to_string(), inverse: (*inverse).to_string(), description: None, is_builtin: true });
            seen.insert(*name); seen.insert(*inverse);
        }
    }
    for custom in custom_types {
        types.push(LinkTypeInfo { name: custom.name.clone(), inverse: custom.inverse.clone(), description: custom.description.clone(), is_builtin: false });
    }
    types
}
#[cfg(test)] #[path = "crud_tests_1.rs"] mod tests_1;
#[cfg(test)] #[path = "crud_tests_2.rs"] mod tests_2;
