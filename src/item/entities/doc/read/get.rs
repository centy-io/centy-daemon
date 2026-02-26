use crate::item::entities::doc::content::read_doc_from_disk;
use crate::item::entities::doc::error::DocError;
use crate::item::entities::doc::types::Doc;
use crate::manifest::read_manifest;
use crate::utils::get_centy_path;
use std::path::Path;
/// Get a single doc by its slug
pub async fn get_doc(project_path: &Path, slug: &str) -> Result<Doc, DocError> {
    read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let doc_path = centy_path.join("docs").join(format!("{slug}.md"));
    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }
    read_doc_from_disk(&doc_path, slug).await
}
