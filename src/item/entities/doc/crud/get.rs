use std::path::Path;

use tokio::fs;

use crate::manifest::read_manifest;
use crate::registry::ProjectInfo;
use crate::utils::get_centy_path;

use super::io::read_doc_from_disk;
use super::options::{DocWithProject, GetDocsBySlugResult};
use super::slug::validate_slug;
use super::types::{Doc, DocError};

/// Get a single doc by its slug
pub async fn get_doc(project_path: &Path, slug: &str) -> Result<Doc, DocError> {
    read_manifest(project_path).await?.ok_or(DocError::NotInitialized)?;
    let doc_path = get_centy_path(project_path).join("docs").join(format!("{slug}.md"));
    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }
    read_doc_from_disk(&doc_path, slug).await
}

/// List all docs
pub async fn list_docs(project_path: &Path, include_deleted: bool) -> Result<Vec<Doc>, DocError> {
    read_manifest(project_path).await?.ok_or(DocError::NotInitialized)?;
    let docs_path = get_centy_path(project_path).join("docs");
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

/// Search for docs by slug across all tracked projects
pub async fn get_docs_by_slug(
    slug: &str,
    projects: &[ProjectInfo],
) -> Result<GetDocsBySlugResult, DocError> {
    validate_slug(slug)?;
    let mut found_docs = Vec::new();
    let mut errors = Vec::new();
    for project in projects {
        if !project.initialized {
            continue;
        }
        let project_path = Path::new(&project.path);
        match get_doc(project_path, slug).await {
            Ok(doc) => {
                let project_name = project.name.clone().unwrap_or_else(|| {
                    project_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| project.path.clone())
                });
                found_docs.push(DocWithProject { doc, project_path: project.path.clone(), project_name });
            }
            Err(DocError::DocNotFound(_) | DocError::NotInitialized) => {}
            Err(e) => errors.push(format!("Error searching {}: {}", project.path, e)),
        }
    }
    Ok(GetDocsBySlugResult { docs: found_docs, errors })
}
