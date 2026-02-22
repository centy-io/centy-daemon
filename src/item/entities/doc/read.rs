use std::path::Path;

use crate::manifest::read_manifest;
use crate::registry::ProjectInfo;
use crate::utils::get_centy_path;
use tokio::fs;

use super::error::DocError;
use super::parse::parse_doc_content;
use super::slug::validate_slug;
use super::types::{Doc, DocWithProject, GetDocsBySlugResult};

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
                // Skip the README.md that's managed by centy
                if slug == "README" {
                    continue;
                }
                if let Ok(doc) = read_doc_from_disk(&path, slug).await {
                    // Filter out soft-deleted unless include_deleted is true
                    if include_deleted || doc.metadata.deleted_at.is_none() {
                        docs.push(doc);
                    }
                }
                // Skip docs that can't be read
            }
        }
    }

    // Sort by slug
    docs.sort_by(|a, b| a.slug.cmp(&b.slug));

    Ok(docs)
}

/// Search for docs by slug across all tracked projects
/// This is a global search that doesn't require a project_path
pub async fn get_docs_by_slug(
    slug: &str,
    projects: &[ProjectInfo],
) -> Result<GetDocsBySlugResult, DocError> {
    // Validate slug format
    validate_slug(slug)?;

    let mut found_docs = Vec::new();
    let mut errors = Vec::new();

    for project in projects {
        // Skip uninitialized projects
        if !project.initialized {
            continue;
        }

        let project_path = Path::new(&project.path);

        // Try to get the doc from this project
        match get_doc(project_path, slug).await {
            Ok(doc) => {
                let project_name = project.name.clone().unwrap_or_else(|| {
                    project_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| project.path.clone())
                });

                found_docs.push(DocWithProject {
                    doc,
                    project_path: project.path.clone(),
                    project_name,
                });
            }
            Err(DocError::DocNotFound(_)) => {
                // Not an error - doc simply doesn't exist in this project
            }
            Err(DocError::NotInitialized) => {
                // Skip - project not properly initialized
            }
            Err(e) => {
                // Log non-fatal errors but continue searching
                errors.push(format!("Error searching {}: {}", project.path, e));
            }
        }
    }

    Ok(GetDocsBySlugResult {
        docs: found_docs,
        errors,
    })
}

/// Read a doc from disk
pub(super) async fn read_doc_from_disk(doc_path: &Path, slug: &str) -> Result<Doc, DocError> {
    let content = fs::read_to_string(doc_path).await?;
    let (title, body, metadata) = parse_doc_content(&content);

    Ok(Doc {
        slug: slug.to_string(),
        title,
        content: body,
        metadata,
    })
}
