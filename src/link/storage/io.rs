#![allow(unknown_lints, max_nesting_depth)]
use super::links_file::{LinksFile, LINKS_FILENAME};
use std::path::Path;
use tokio::fs;
/// Read links from an entity's folder.
/// Returns an empty `LinksFile` if the file doesn't exist.
pub async fn read_links(entity_path: &Path) -> Result<LinksFile, std::io::Error> {
    let old_links_path = entity_path.join(LINKS_FILENAME);
    if old_links_path.exists() {
        let content = fs::read_to_string(&old_links_path).await?;
        let links_file: LinksFile = serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse links.json: {e}"))
        })?;
        return Ok(links_file);
    }
    if let (Some(parent), Some(entity_id)) = (entity_path.parent(), entity_path.file_name()) {
        let new_links_path = parent.join("links").join(entity_id).join(LINKS_FILENAME);
        if new_links_path.exists() {
            let content = fs::read_to_string(&new_links_path).await?;
            let links_file: LinksFile = serde_json::from_str(&content).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse links.json: {e}"))
            })?;
            return Ok(links_file);
        }
    }
    Ok(LinksFile::new())
}
/// Write links to an entity's links folder.
/// Uses new format: parent/links/{entity_id}/links.json.
/// If the links list is empty, deletes the file if it exists.
pub async fn write_links(entity_path: &Path, links_file: &LinksFile) -> Result<(), std::io::Error> {
    let (parent, entity_id) = match (entity_path.parent(), entity_path.file_name()) {
        (Some(p), Some(id)) => (p, id),
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid entity path")),
    };
    let links_dir = parent.join("links").join(entity_id);
    let new_links_path = links_dir.join(LINKS_FILENAME);
    if links_file.links.is_empty() {
        if new_links_path.exists() { fs::remove_file(&new_links_path).await?; }
        let old_links_path = entity_path.join(LINKS_FILENAME);
        if old_links_path.exists() { fs::remove_file(&old_links_path).await?; }
        if links_dir.exists()
            && fs::read_dir(&links_dir).await?.next_entry().await?.is_none()
        {
            let _ = fs::remove_dir(&links_dir).await;
        }
        return Ok(());
    }
    fs::create_dir_all(&links_dir).await?;
    let content = serde_json::to_string_pretty(links_file).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to serialize links: {e}"))
    })?;
    fs::write(&new_links_path, content).await?;
    let old_links_path = entity_path.join(LINKS_FILENAME);
    if old_links_path.exists() { let _ = fs::remove_file(&old_links_path).await; }
    Ok(())
}
