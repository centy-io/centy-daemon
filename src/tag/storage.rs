//! Tag storage operations for reading/writing tags.json.

use super::types::{Tag, TagError, TagsFile};
use crate::manifest::read_manifest;
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;

/// Read tags from the .centy/tags.json file.
/// Returns an empty list if the file doesn't exist.
pub async fn read_tags(project_path: &Path) -> Result<Vec<Tag>, TagError> {
    // Verify project is initialized
    read_manifest(project_path)
        .await
        .map_err(|_| TagError::NotInitialized)?
        .ok_or(TagError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let tags_path = centy_path.join("tags.json");

    if !tags_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&tags_path).await?;
    let tags_file: TagsFile = serde_json::from_str(&content)?;

    Ok(tags_file.tags)
}

/// Write tags to the .centy/tags.json file.
/// Tags are sorted by name (A-Z), then by created_at.
pub async fn write_tags(project_path: &Path, tags: &[Tag]) -> Result<(), TagError> {
    let centy_path = get_centy_path(project_path);
    let tags_path = centy_path.join("tags.json");

    // Sort tags: A-Z by name, then by created_at
    let mut sorted_tags = tags.to_vec();
    sorted_tags.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });

    let tags_file = TagsFile { tags: sorted_tags };

    let content = serde_json::to_string_pretty(&tags_file)?;
    fs::write(&tags_path, content).await?;

    Ok(())
}

/// Find a tag by name in a list of tags.
pub fn find_tag_by_name<'a>(tags: &'a [Tag], name: &str) -> Option<&'a Tag> {
    tags.iter().find(|t| t.name == name)
}

/// Find the index of a tag by name in a list of tags.
pub fn find_tag_index_by_name(tags: &[Tag], name: &str) -> Option<usize> {
    tags.iter().position(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_tag_by_name() {
        let tags = vec![
            Tag {
                name: "bug".to_string(),
                color: Some("#ef4444".to_string()),
                created_at: "2025-01-01T00:00:00Z".to_string(),
                is_org_tag: false,
                org_slug: None,
            },
            Tag {
                name: "feature".to_string(),
                color: None,
                created_at: "2025-01-02T00:00:00Z".to_string(),
                is_org_tag: false,
                org_slug: None,
            },
        ];

        assert!(find_tag_by_name(&tags, "bug").is_some());
        assert!(find_tag_by_name(&tags, "feature").is_some());
        assert!(find_tag_by_name(&tags, "nonexistent").is_none());
    }

    #[test]
    fn test_find_tag_index_by_name() {
        let tags = vec![
            Tag {
                name: "bug".to_string(),
                color: None,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                is_org_tag: false,
                org_slug: None,
            },
            Tag {
                name: "feature".to_string(),
                color: None,
                created_at: "2025-01-02T00:00:00Z".to_string(),
                is_org_tag: false,
                org_slug: None,
            },
        ];

        assert_eq!(find_tag_index_by_name(&tags, "bug"), Some(0));
        assert_eq!(find_tag_index_by_name(&tags, "feature"), Some(1));
        assert_eq!(find_tag_index_by_name(&tags, "nonexistent"), None);
    }

    #[test]
    #[allow(clippy::useless_vec)]
    fn test_tags_sorting() {
        let mut tags = vec![
            Tag {
                name: "zebra".to_string(),
                color: None,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                is_org_tag: false,
                org_slug: None,
            },
            Tag {
                name: "alpha".to_string(),
                color: None,
                created_at: "2025-01-02T00:00:00Z".to_string(),
                is_org_tag: false,
                org_slug: None,
            },
            Tag {
                name: "beta".to_string(),
                color: None,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                is_org_tag: false,
                org_slug: None,
            },
        ];

        // Sort like write_tags does
        tags.sort_by(|a, b| {
            a.name
                .cmp(&b.name)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        assert_eq!(tags[0].name, "alpha");
        assert_eq!(tags[1].name, "beta");
        assert_eq!(tags[2].name, "zebra");
    }
}
