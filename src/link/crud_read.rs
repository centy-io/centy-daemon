use super::crud_helpers::{entity_exists, get_entity_path};
use super::crud_types::{LinkError, LinkTypeInfo};
use super::{read_links, CustomLinkTypeDefinition, TargetType, BUILTIN_LINK_TYPES};
use std::path::Path;

pub async fn list_links(
    project_path: &Path,
    entity_id: &str,
    entity_type: TargetType,
) -> Result<super::LinksFile, LinkError> {
    if !entity_exists(project_path, entity_id, &entity_type) {
        return Err(LinkError::SourceNotFound(
            entity_id.to_string(),
            entity_type,
        ));
    }
    let entity_path = get_entity_path(project_path, entity_id, &entity_type);
    Ok(read_links(&entity_path).await?)
}

#[must_use] 
pub fn get_available_link_types(custom_types: &[CustomLinkTypeDefinition]) -> Vec<LinkTypeInfo> {
    let mut types = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (name, inverse) in BUILTIN_LINK_TYPES {
        if !seen.contains(name) && !seen.contains(inverse) {
            types.push(LinkTypeInfo {
                name: (*name).to_string(),
                inverse: (*inverse).to_string(),
                description: None,
                is_builtin: true,
            });
            seen.insert(*name);
            seen.insert(*inverse);
        }
    }
    for custom in custom_types {
        types.push(LinkTypeInfo {
            name: custom.name.clone(),
            inverse: custom.inverse.clone(),
            description: custom.description.clone(),
            is_builtin: false,
        });
    }
    types
}
