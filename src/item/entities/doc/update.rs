use super::content::{generate_doc_content, read_doc_from_disk};
use super::error::DocError;
use super::helpers::{slugify, validate_slug};
use super::types::{Doc, DocMetadata, OrgDocSyncResult, UpdateDocOptions, UpdateDocResult};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::registry::get_org_projects;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;
/// Update an existing doc
pub async fn update_doc(
    project_path: &Path,
    slug: &str,
    options: UpdateDocOptions,
) -> Result<UpdateDocResult, DocError> {
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");
    let doc_path = docs_path.join(format!("{slug}.md"));
    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }
    let current = read_doc_from_disk(&doc_path, slug).await?;
    let new_title = options.title.unwrap_or(current.title);
    let new_content = options.content.unwrap_or(current.content);
    let new_slug = match options.new_slug {
        Some(s) if !s.trim().is_empty() && s != slug => {
            let new_slug = slugify(&s);
            validate_slug(&new_slug)?;
            let new_path = docs_path.join(format!("{new_slug}.md"));
            if new_path.exists() {
                return Err(DocError::SlugAlreadyExists(new_slug));
            }
            Some(new_slug)
        }
        _ => None,
    };
    let updated_metadata = DocMetadata {
        created_at: current.metadata.created_at.clone(),
        updated_at: now_iso(),
        deleted_at: current.metadata.deleted_at.clone(),
        is_org_doc: current.metadata.is_org_doc,
        org_slug: current.metadata.org_slug.clone(),
    };
    let doc_content = generate_doc_content(&new_title, &new_content, &updated_metadata);
    let final_slug = if let Some(ref new_slug) = new_slug {
        fs::remove_file(&doc_path).await?;
        fs::write(docs_path.join(format!("{new_slug}.md")), &doc_content).await?;
        new_slug.clone()
    } else {
        fs::write(&doc_path, &doc_content).await?;
        slug.to_string()
    };
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    let doc = Doc {
        slug: final_slug.clone(),
        title: new_title.clone(),
        content: new_content.clone(),
        metadata: updated_metadata,
    };
    let sync_results = if doc.metadata.is_org_doc {
        if let Some(ref org) = doc.metadata.org_slug {
            sync_org_doc_update_to_projects(
                org,
                project_path,
                &final_slug,
                &new_title,
                &new_content,
                new_slug.as_ref().map(|_| slug),
            )
            .await
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    Ok(UpdateDocResult {
        doc,
        manifest,
        sync_results,
    })
}
/// Sync an org doc update to all other projects in the organization
pub async fn sync_org_doc_update_to_projects(
    org_slug: &str,
    source_project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
    old_slug: Option<&str>,
) -> Vec<OrgDocSyncResult> {
    let source_path_str = source_project_path.to_string_lossy().to_string();
    let org_projects = match get_org_projects(org_slug, Some(&source_path_str)).await {
        Ok(projects) => projects,
        Err(e) => {
            return vec![OrgDocSyncResult {
                project_path: "<registry>".to_string(),
                success: false,
                error: Some(format!("Failed to get org projects: {e}")),
            }]
        }
    };
    let mut results = Vec::new();
    for project in org_projects {
        let result = update_or_create_doc_in_project(
            Path::new(&project.path),
            slug,
            title,
            content,
            org_slug,
            old_slug,
        )
        .await;
        results.push(OrgDocSyncResult {
            project_path: project.path.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        });
    }
    results
}
/// Update or create a doc in a specific project (used for org doc sync on update)
#[allow(unknown_lints, max_nesting_depth)]
pub async fn update_or_create_doc_in_project(
    project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
    org_slug: &str,
    old_slug: Option<&str>,
) -> Result<(), DocError> {
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");
    if !docs_path.exists() {
        fs::create_dir_all(&docs_path).await?;
    }
    let doc_path = docs_path.join(format!("{slug}.md"));
    if let Some(old) = old_slug {
        if old != slug {
            let old_doc_path = docs_path.join(format!("{old}.md"));
            if old_doc_path.exists() {
                fs::remove_file(&old_doc_path).await?;
            }
        }
    }
    let metadata = if doc_path.exists() {
        let existing = read_doc_from_disk(&doc_path, slug).await?;
        DocMetadata {
            created_at: existing.metadata.created_at,
            updated_at: now_iso(),
            deleted_at: None,
            is_org_doc: true,
            org_slug: Some(org_slug.to_string()),
        }
    } else {
        let old_created_at = if let Some(old) = old_slug {
            let old_doc_path = docs_path.join(format!("{old}.md"));
            #[allow(unknown_lints, max_nesting_depth)]
            if old_doc_path.exists() {
                read_doc_from_disk(&old_doc_path, old)
                    .await
                    .ok()
                    .map(|d| d.metadata.created_at)
            } else {
                None
            }
        } else {
            None
        };
        DocMetadata {
            created_at: old_created_at.unwrap_or_else(now_iso),
            updated_at: now_iso(),
            deleted_at: None,
            is_org_doc: true,
            org_slug: Some(org_slug.to_string()),
        }
    };
    fs::write(
        &doc_path,
        crate::utils::format_markdown(&generate_doc_content(title, content, &metadata)),
    )
    .await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    Ok(())
}
