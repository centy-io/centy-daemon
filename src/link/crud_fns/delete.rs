use super::super::crud_helpers::get_entity_path;
use super::super::crud_types::{DeleteLinkOptions, DeleteLinkResult, LinkError};
use super::super::{get_inverse_link_type, read_links, write_links, CustomLinkTypeDefinition};
use std::path::Path;
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
            .map(|l| l.kind.clone())
            .collect();
        if source_links.remove_link(&options.target_id, None) {
            deleted_count =
                deleted_count.saturating_add(link_types.len().try_into().unwrap_or(u32::MAX));
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
