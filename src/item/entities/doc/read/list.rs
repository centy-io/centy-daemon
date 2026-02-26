use crate::item::entities::doc::content::read_doc_from_disk;
use crate::item::entities::doc::error::DocError;
use crate::item::entities::doc::types::Doc;
use crate::manifest::read_manifest;
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;
/// List all docs
#[allow(unknown_lints, max_nesting_depth)]
pub async fn list_docs(project_path: &Path, include_deleted: bool) -> Result<Vec<Doc>, DocError> {
    read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");
    if !docs_path.exists() {
        return Ok(Vec::new());
    }
    let mut docs = Vec::new();
    let mut entries = fs::read_dir(&docs_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            if let Some(slug) = path.file_stem().and_then(|s| s.to_str()) {
                if slug == "README" {
                    continue;
                }
                if let Ok(doc) = read_doc_from_disk(&path, slug).await {
                    if include_deleted || doc.metadata.deleted_at.is_none() {
                        docs.push(doc);
                    }
                }
            }
        }
    }
    docs.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(docs)
}
