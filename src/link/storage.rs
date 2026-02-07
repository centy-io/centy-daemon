use super::Link;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

/// The filename for links storage
pub const LINKS_FILENAME: &str = "links.json";

/// Container for links stored in links.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LinksFile {
    /// The list of links for this entity
    #[serde(default)]
    pub links: Vec<Link>,
}

impl LinksFile {
    /// Create a new empty links file
    #[must_use]
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }

    /// Add a link to this file
    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    /// Remove a link matching the target and link type
    ///
    /// Returns `true` if a link was removed, `false` otherwise.
    pub fn remove_link(&mut self, target_id: &str, link_type: Option<&str>) -> bool {
        let initial_len = self.links.len();
        self.links.retain(|link| {
            if link.target_id != target_id {
                return true;
            }
            if let Some(lt) = link_type {
                link.link_type != lt
            } else {
                false // Remove all links to this target
            }
        });
        self.links.len() < initial_len
    }

    /// Check if a link exists
    pub fn has_link(&self, target_id: &str, link_type: &str) -> bool {
        self.links
            .iter()
            .any(|link| link.target_id == target_id && link.link_type == link_type)
    }
}

/// Read links from an entity's folder
///
/// Supports both old format (links.json in entity folder) and new format
/// (links.json in parent/links/{entity_id}/ folder).
///
/// Returns an empty `LinksFile` if the file doesn't exist.
pub async fn read_links(entity_path: &Path) -> Result<LinksFile, std::io::Error> {
    // Try old format first: entity_path/links.json (for folders)
    let old_links_path = entity_path.join(LINKS_FILENAME);
    if old_links_path.exists() {
        let content = fs::read_to_string(&old_links_path).await?;
        let links_file: LinksFile = serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse links.json: {e}"),
            )
        })?;
        return Ok(links_file);
    }

    // Try new format: parent/links/{entity_id}/links.json
    // entity_path = .centy/issues/{entity_id}
    // new_links_path = .centy/issues/links/{entity_id}/links.json
    if let (Some(parent), Some(entity_id)) = (entity_path.parent(), entity_path.file_name()) {
        let new_links_path = parent.join("links").join(entity_id).join(LINKS_FILENAME);
        if new_links_path.exists() {
            let content = fs::read_to_string(&new_links_path).await?;
            let links_file: LinksFile = serde_json::from_str(&content).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse links.json: {e}"),
                )
            })?;
            return Ok(links_file);
        }
    }

    Ok(LinksFile::new())
}

/// Write links to an entity's links folder
///
/// Uses new format: parent/links/{entity_id}/links.json
/// Creates the file if it doesn't exist.
/// If the links list is empty, deletes the file if it exists.
pub async fn write_links(entity_path: &Path, links_file: &LinksFile) -> Result<(), std::io::Error> {
    // Use new format: parent/links/{entity_id}/links.json
    let (parent, entity_id) = match (entity_path.parent(), entity_path.file_name()) {
        (Some(p), Some(id)) => (p, id),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid entity path",
            ))
        }
    };

    let links_dir = parent.join("links").join(entity_id);
    let new_links_path = links_dir.join(LINKS_FILENAME);

    // If no links, remove the file if it exists
    if links_file.links.is_empty() {
        if new_links_path.exists() {
            fs::remove_file(&new_links_path).await?;
        }
        // Also clean up old format if it exists
        let old_links_path = entity_path.join(LINKS_FILENAME);
        if old_links_path.exists() {
            fs::remove_file(&old_links_path).await?;
        }
        // Clean up empty links directory
        if links_dir.exists()
            && fs::read_dir(&links_dir)
                .await?
                .next_entry()
                .await?
                .is_none()
        {
            let _ = fs::remove_dir(&links_dir).await;
        }
        return Ok(());
    }

    // Create links directory if needed
    fs::create_dir_all(&links_dir).await?;

    let content = serde_json::to_string_pretty(links_file).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to serialize links: {e}"),
        )
    })?;

    fs::write(&new_links_path, content).await?;

    // Clean up old format if it exists
    let old_links_path = entity_path.join(LINKS_FILENAME);
    if old_links_path.exists() {
        let _ = fs::remove_file(&old_links_path).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::TargetType;

    #[test]
    fn test_links_file_new() {
        let file = LinksFile::new();
        assert!(file.links.is_empty());
    }

    #[test]
    fn test_links_file_add_link() {
        let mut file = LinksFile::new();
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));
        assert_eq!(file.links.len(), 1);
    }

    #[test]
    fn test_links_file_remove_link() {
        let mut file = LinksFile::new();
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "parent-of".to_string(),
        ));

        // Remove specific link type
        assert!(file.remove_link("uuid-1", Some("blocks")));
        assert_eq!(file.links.len(), 1);
        assert_eq!(file.links[0].link_type, "parent-of");
    }

    #[test]
    fn test_links_file_remove_all_links_to_target() {
        let mut file = LinksFile::new();
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "parent-of".to_string(),
        ));
        file.add_link(Link::new(
            "uuid-2".to_string(),
            TargetType::Doc,
            "relates-to".to_string(),
        ));

        // Remove all links to uuid-1
        assert!(file.remove_link("uuid-1", None));
        assert_eq!(file.links.len(), 1);
        assert_eq!(file.links[0].target_id, "uuid-2");
    }

    #[test]
    fn test_links_file_has_link() {
        let mut file = LinksFile::new();
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));

        assert!(file.has_link("uuid-1", "blocks"));
        assert!(!file.has_link("uuid-1", "parent-of"));
        assert!(!file.has_link("uuid-2", "blocks"));
    }

    #[test]
    fn test_links_file_serialization() {
        let mut file = LinksFile::new();
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));

        let json = serde_json::to_string_pretty(&file).unwrap();
        assert!(json.contains("\"links\""));
        assert!(json.contains("\"targetId\": \"uuid-1\""));

        let parsed: LinksFile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.links.len(), 1);
    }

    #[test]
    fn test_links_file_default() {
        let file = LinksFile::default();
        assert!(file.links.is_empty());
    }

    #[test]
    fn test_links_file_remove_nonexistent() {
        let mut file = LinksFile::new();
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));

        // Try to remove a link that doesn't exist
        assert!(!file.remove_link("uuid-999", Some("blocks")));
        assert_eq!(file.links.len(), 1);
    }

    #[test]
    fn test_links_file_has_link_empty() {
        let file = LinksFile::new();
        assert!(!file.has_link("any-id", "any-type"));
    }

    #[test]
    fn test_links_file_multiple_links() {
        let mut file = LinksFile::new();
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));
        file.add_link(Link::new(
            "uuid-2".to_string(),
            TargetType::Doc,
            "relates-to".to_string(),
        ));
        file.add_link(Link::new(
            "uuid-3".to_string(),
            TargetType::Pr,
            "parent-of".to_string(),
        ));

        assert_eq!(file.links.len(), 3);
        assert!(file.has_link("uuid-1", "blocks"));
        assert!(file.has_link("uuid-2", "relates-to"));
        assert!(file.has_link("uuid-3", "parent-of"));
    }

    #[test]
    fn test_links_file_clone() {
        let mut file = LinksFile::new();
        file.add_link(Link::new(
            "uuid-1".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));

        let cloned = file.clone();
        assert_eq!(cloned.links.len(), 1);
        assert_eq!(cloned.links[0].target_id, "uuid-1");
    }

    #[test]
    fn test_links_filename_constant() {
        assert_eq!(LINKS_FILENAME, "links.json");
    }

    #[tokio::test]
    async fn test_read_links_nonexistent() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let entity_path = temp_dir.path().join("issues").join("uuid-1");

        let links = read_links(&entity_path).await.expect("Should read");
        assert!(links.links.is_empty());
    }

    #[tokio::test]
    async fn test_write_and_read_links() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let issues_path = temp_dir.path().join("issues");
        fs::create_dir_all(&issues_path)
            .await
            .expect("Should create dirs");

        let entity_path = issues_path.join("uuid-1");

        let mut links_file = LinksFile::new();
        links_file.add_link(Link::new(
            "uuid-2".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));

        write_links(&entity_path, &links_file)
            .await
            .expect("Should write");

        let read_back = read_links(&entity_path).await.expect("Should read");
        assert_eq!(read_back.links.len(), 1);
        assert_eq!(read_back.links[0].target_id, "uuid-2");
        assert_eq!(read_back.links[0].link_type, "blocks");
    }

    #[tokio::test]
    async fn test_write_empty_links_removes_file() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let issues_path = temp_dir.path().join("issues");
        fs::create_dir_all(&issues_path)
            .await
            .expect("Should create dirs");

        let entity_path = issues_path.join("uuid-1");

        // Write a link first
        let mut links_file = LinksFile::new();
        links_file.add_link(Link::new(
            "uuid-2".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        ));
        write_links(&entity_path, &links_file)
            .await
            .expect("Should write");

        // Write empty links - should clean up
        let empty_file = LinksFile::new();
        write_links(&entity_path, &empty_file)
            .await
            .expect("Should write empty");

        // Reading should return empty
        let read_back = read_links(&entity_path).await.expect("Should read");
        assert!(read_back.links.is_empty());
    }
}
