use super::crud_types::{LinkError, LinkTypeInfo};
use super::storage::list_all_link_records;
use super::types::{CustomLinkTypeDefinition, LinkView, TargetType};
use super::BUILTIN_LINK_TYPES;
use std::path::Path;

/// List all links for an entity, returning a view for each (with direction set).
pub async fn list_links(
    project_path: &Path,
    entity_id: &str,
    _entity_type: TargetType,
) -> Result<Vec<LinkView>, LinkError> {
    let all_records = list_all_link_records(project_path).await?;
    let mut views = Vec::new();
    for record in all_records {
        if record.source_id == entity_id {
            views.push(record.source_view());
        } else if record.target_id == entity_id {
            views.push(record.target_view());
        } else {
            // Link involves neither source nor target — skip.
        }
    }
    Ok(views)
}

#[must_use]
pub fn get_available_link_types(custom_types: &[CustomLinkTypeDefinition]) -> Vec<LinkTypeInfo> {
    let mut types: Vec<LinkTypeInfo> = BUILTIN_LINK_TYPES
        .iter()
        .map(|&name| LinkTypeInfo {
            name: name.to_string(),
            description: None,
            is_builtin: true,
        })
        .collect();
    for custom in custom_types {
        types.push(LinkTypeInfo {
            name: custom.name.clone(),
            description: custom.description.clone(),
            is_builtin: false,
        });
    }
    types
}
