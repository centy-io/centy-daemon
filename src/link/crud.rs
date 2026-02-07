use super::{
    get_inverse_link_type, is_valid_link_type, read_links, write_links, CustomLinkTypeDefinition,
    Link, LinksFile, TargetType, BUILTIN_LINK_TYPES,
};
use crate::utils::get_centy_path;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LinkError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid link type: {0}")]
    InvalidLinkType(String),

    #[error("Source entity not found: {0} ({1})")]
    SourceNotFound(String, TargetType),

    #[error("Target entity not found: {0} ({1})")]
    TargetNotFound(String, TargetType),

    #[error("Link already exists")]
    LinkAlreadyExists,

    #[error("Link not found")]
    LinkNotFound,

    #[error("Cannot link entity to itself")]
    SelfLink,
}

/// Options for creating a link
#[derive(Debug, Clone)]
pub struct CreateLinkOptions {
    pub source_id: String,
    pub source_type: TargetType,
    pub target_id: String,
    pub target_type: TargetType,
    pub link_type: String,
}

/// Result of creating a link
#[derive(Debug)]
pub struct CreateLinkResult {
    pub created_link: Link,
    pub inverse_link: Link,
}

/// Options for deleting a link
#[derive(Debug, Clone)]
pub struct DeleteLinkOptions {
    pub source_id: String,
    pub source_type: TargetType,
    pub target_id: String,
    pub target_type: TargetType,
    /// If provided, only delete links of this type
    /// If None, delete all links between source and target
    pub link_type: Option<String>,
}

/// Result of deleting a link
#[derive(Debug)]
pub struct DeleteLinkResult {
    /// Number of links deleted (including inverse links)
    pub deleted_count: u32,
}

/// Information about a link type
#[derive(Debug, Clone)]
pub struct LinkTypeInfo {
    pub name: String,
    pub inverse: String,
    pub description: Option<String>,
    pub is_builtin: bool,
}

/// Get the path to an entity's folder
fn get_entity_path(
    project_path: &Path,
    entity_id: &str,
    entity_type: TargetType,
) -> std::path::PathBuf {
    get_centy_path(project_path)
        .join(entity_type.folder_name())
        .join(entity_id)
}

/// Check if an entity exists
/// Supports both new format (.md file) and old format (folder)
async fn entity_exists(project_path: &Path, entity_id: &str, entity_type: TargetType) -> bool {
    let centy_path = get_centy_path(project_path);
    let base_path = centy_path.join(entity_type.folder_name());

    // Check for new format: {id}.md file
    let file_path = base_path.join(format!("{entity_id}.md"));
    if file_path.exists() {
        return true;
    }

    // Check for old format: {id}/ folder
    let folder_path = base_path.join(entity_id);
    folder_path.exists()
}

/// Create a link between two entities
///
/// This creates both the forward link and the inverse link atomically.
pub async fn create_link(
    project_path: &Path,
    options: CreateLinkOptions,
    custom_types: &[CustomLinkTypeDefinition],
) -> Result<CreateLinkResult, LinkError> {
    // Validate link type
    if !is_valid_link_type(&options.link_type, custom_types) {
        return Err(LinkError::InvalidLinkType(options.link_type));
    }

    // Prevent self-links
    if options.source_id == options.target_id && options.source_type == options.target_type {
        return Err(LinkError::SelfLink);
    }

    // Check source exists
    if !entity_exists(project_path, &options.source_id, options.source_type).await {
        return Err(LinkError::SourceNotFound(
            options.source_id.clone(),
            options.source_type,
        ));
    }

    // Check target exists
    if !entity_exists(project_path, &options.target_id, options.target_type).await {
        return Err(LinkError::TargetNotFound(
            options.target_id.clone(),
            options.target_type,
        ));
    }

    // Get inverse link type
    let inverse_type = get_inverse_link_type(&options.link_type, custom_types)
        .ok_or_else(|| LinkError::InvalidLinkType(options.link_type.clone()))?;

    // Get paths
    let source_path = get_entity_path(project_path, &options.source_id, options.source_type);
    let target_path = get_entity_path(project_path, &options.target_id, options.target_type);

    // Read existing links
    let mut source_links = read_links(&source_path).await?;
    let mut target_links = read_links(&target_path).await?;

    // Check if link already exists
    if source_links.has_link(&options.target_id, &options.link_type) {
        return Err(LinkError::LinkAlreadyExists);
    }

    // Create forward link
    let forward_link = Link::new(
        options.target_id.clone(),
        options.target_type,
        options.link_type.clone(),
    );

    // Create inverse link
    let inverse_link = Link::new(options.source_id.clone(), options.source_type, inverse_type);

    // Add links
    source_links.add_link(forward_link.clone());
    target_links.add_link(inverse_link.clone());

    // Write both files atomically (best effort - not truly atomic)
    write_links(&source_path, &source_links).await?;
    write_links(&target_path, &target_links).await?;

    Ok(CreateLinkResult {
        created_link: forward_link,
        inverse_link,
    })
}

/// Delete a link between two entities
///
/// This deletes both the forward link and the inverse link.
pub async fn delete_link(
    project_path: &Path,
    options: DeleteLinkOptions,
    custom_types: &[CustomLinkTypeDefinition],
) -> Result<DeleteLinkResult, LinkError> {
    let source_path = get_entity_path(project_path, &options.source_id, options.source_type);
    let target_path = get_entity_path(project_path, &options.target_id, options.target_type);

    // Read existing links
    let mut source_links = read_links(&source_path).await?;
    let mut target_links = read_links(&target_path).await?;

    let mut deleted_count = 0u32;

    if let Some(link_type) = &options.link_type {
        // Delete specific link type
        if source_links.remove_link(&options.target_id, Some(link_type)) {
            deleted_count = deleted_count.saturating_add(1);
        }

        // Delete inverse link
        if let Some(inverse_type) = get_inverse_link_type(link_type, custom_types) {
            if target_links.remove_link(&options.source_id, Some(&inverse_type)) {
                deleted_count = deleted_count.saturating_add(1);
            }
        }
    } else {
        // Delete all links between source and target
        // First, find all link types from source to target
        let link_types: Vec<String> = source_links
            .links
            .iter()
            .filter(|l| l.target_id == options.target_id)
            .map(|l| l.link_type.clone())
            .collect();

        // Remove forward links
        if source_links.remove_link(&options.target_id, None) {
            deleted_count = deleted_count.saturating_add(link_types.len() as u32);
        }

        // Remove inverse links for each type
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

    // Write both files
    write_links(&source_path, &source_links).await?;
    write_links(&target_path, &target_links).await?;

    Ok(DeleteLinkResult { deleted_count })
}

/// List all links for an entity
pub async fn list_links(
    project_path: &Path,
    entity_id: &str,
    entity_type: TargetType,
) -> Result<LinksFile, LinkError> {
    // Check if entity exists (supports both old and new formats)
    if !entity_exists(project_path, entity_id, entity_type).await {
        return Err(LinkError::SourceNotFound(
            entity_id.to_string(),
            entity_type,
        ));
    }

    // Use entity_path for reading links (read_links handles both formats)
    let entity_path = get_entity_path(project_path, entity_id, entity_type);
    let links = read_links(&entity_path).await?;
    Ok(links)
}

/// Get all available link types (builtin + custom)
pub fn get_available_link_types(custom_types: &[CustomLinkTypeDefinition]) -> Vec<LinkTypeInfo> {
    let mut types = Vec::new();

    // Add builtin types (deduplicated - only add forward direction)
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

    // Add custom types
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_link_types_builtin() {
        let custom: Vec<CustomLinkTypeDefinition> = vec![];
        let types = get_available_link_types(&custom);

        // Should have 4 builtin pairs (blocks, parent-of, relates-to, duplicates)
        assert_eq!(types.len(), 4);
        assert!(types.iter().all(|t| t.is_builtin));
    }

    #[test]
    fn test_get_available_link_types_with_custom() {
        let custom = vec![CustomLinkTypeDefinition {
            name: "depends-on".to_string(),
            inverse: "dependency-of".to_string(),
            description: Some("Dependency relationship".to_string()),
        }];
        let types = get_available_link_types(&custom);

        // Should have 4 builtin + 1 custom
        assert_eq!(types.len(), 5);

        let custom_type = types.iter().find(|t| !t.is_builtin).unwrap();
        assert_eq!(custom_type.name, "depends-on");
        assert_eq!(custom_type.inverse, "dependency-of");
        assert_eq!(
            custom_type.description,
            Some("Dependency relationship".to_string())
        );
    }

    #[test]
    fn test_link_error_invalid_link_type() {
        let err = LinkError::InvalidLinkType("unknown-type".to_string());
        let display = format!("{err}");
        assert!(display.contains("Invalid link type"));
        assert!(display.contains("unknown-type"));
    }

    #[test]
    fn test_link_error_source_not_found() {
        let err = LinkError::SourceNotFound("issue-123".to_string(), TargetType::Issue);
        let display = format!("{err}");
        assert!(display.contains("Source entity not found"));
        assert!(display.contains("issue-123"));
    }

    #[test]
    fn test_link_error_target_not_found() {
        let err = LinkError::TargetNotFound("doc-slug".to_string(), TargetType::Doc);
        let display = format!("{err}");
        assert!(display.contains("Target entity not found"));
        assert!(display.contains("doc-slug"));
    }

    #[test]
    fn test_link_error_already_exists() {
        let err = LinkError::LinkAlreadyExists;
        assert_eq!(format!("{err}"), "Link already exists");
    }

    #[test]
    fn test_link_error_not_found() {
        let err = LinkError::LinkNotFound;
        assert_eq!(format!("{err}"), "Link not found");
    }

    #[test]
    fn test_link_error_self_link() {
        let err = LinkError::SelfLink;
        assert_eq!(format!("{err}"), "Cannot link entity to itself");
    }

    #[test]
    fn test_link_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = LinkError::from(io_err);
        assert!(matches!(err, LinkError::IoError(_)));
    }

    #[test]
    fn test_create_link_options_debug() {
        let opts = CreateLinkOptions {
            source_id: "abc".to_string(),
            source_type: TargetType::Issue,
            target_id: "def".to_string(),
            target_type: TargetType::Doc,
            link_type: "blocks".to_string(),
        };
        let debug = format!("{opts:?}");
        assert!(debug.contains("CreateLinkOptions"));
        assert!(debug.contains("abc"));
        assert!(debug.contains("def"));
    }

    #[test]
    fn test_delete_link_options_debug() {
        let opts = DeleteLinkOptions {
            source_id: "abc".to_string(),
            source_type: TargetType::Issue,
            target_id: "def".to_string(),
            target_type: TargetType::Doc,
            link_type: Some("blocks".to_string()),
        };
        let debug = format!("{opts:?}");
        assert!(debug.contains("DeleteLinkOptions"));
    }

    #[test]
    fn test_delete_link_options_without_type() {
        let opts = DeleteLinkOptions {
            source_id: "abc".to_string(),
            source_type: TargetType::Issue,
            target_id: "def".to_string(),
            target_type: TargetType::Doc,
            link_type: None,
        };
        assert!(opts.link_type.is_none());
    }

    #[test]
    fn test_link_type_info_debug() {
        let info = LinkTypeInfo {
            name: "blocks".to_string(),
            inverse: "blocked-by".to_string(),
            description: None,
            is_builtin: true,
        };
        let debug = format!("{info:?}");
        assert!(debug.contains("LinkTypeInfo"));
        assert!(debug.contains("blocks"));
    }

    #[test]
    fn test_link_type_info_clone() {
        let info = LinkTypeInfo {
            name: "blocks".to_string(),
            inverse: "blocked-by".to_string(),
            description: Some("Blocking relationship".to_string()),
            is_builtin: true,
        };
        let cloned = info.clone();
        assert_eq!(cloned.name, "blocks");
        assert_eq!(cloned.inverse, "blocked-by");
        assert!(cloned.is_builtin);
    }

    #[test]
    fn test_get_available_link_types_multiple_custom() {
        let custom = vec![
            CustomLinkTypeDefinition {
                name: "depends-on".to_string(),
                inverse: "dependency-of".to_string(),
                description: None,
            },
            CustomLinkTypeDefinition {
                name: "follows".to_string(),
                inverse: "preceded-by".to_string(),
                description: Some("Sequence order".to_string()),
            },
        ];
        let types = get_available_link_types(&custom);
        assert_eq!(types.len(), 6); // 4 builtin + 2 custom
    }
}
