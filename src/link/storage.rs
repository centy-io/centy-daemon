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

    /// Create a links file with the given links
    #[must_use]
    pub fn with_links(links: Vec<Link>) -> Self {
        Self { links }
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

    /// Get all links of a specific type
    pub fn links_of_type(&self, link_type: &str) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|link| link.link_type == link_type)
            .collect()
    }
}

/// Read links from an entity's folder
///
/// Returns an empty `LinksFile` if the file doesn't exist.
pub async fn read_links(entity_path: &Path) -> Result<LinksFile, std::io::Error> {
    let links_path = entity_path.join(LINKS_FILENAME);

    if !links_path.exists() {
        return Ok(LinksFile::new());
    }

    let content = fs::read_to_string(&links_path).await?;
    let links_file: LinksFile = serde_json::from_str(&content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse links.json: {e}"),
        )
    })?;

    Ok(links_file)
}

/// Write links to an entity's folder
///
/// Creates the file if it doesn't exist.
/// If the links list is empty, deletes the file if it exists.
pub async fn write_links(entity_path: &Path, links_file: &LinksFile) -> Result<(), std::io::Error> {
    let links_path = entity_path.join(LINKS_FILENAME);

    // If no links, remove the file if it exists
    if links_file.links.is_empty() {
        if links_path.exists() {
            fs::remove_file(&links_path).await?;
        }
        return Ok(());
    }

    let content = serde_json::to_string_pretty(links_file).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to serialize links: {e}"),
        )
    })?;

    fs::write(&links_path, content).await?;
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
}
