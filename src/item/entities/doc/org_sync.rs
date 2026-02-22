use std::path::Path;

use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::registry::get_org_projects;
use crate::utils::{format_markdown, get_centy_path, now_iso};
use tokio::fs;

use super::error::DocError;
use super::metadata::DocMetadata;
use super::parse::generate_doc_content;
use super::read::read_doc_from_disk;
use super::types::OrgDocSyncResult;

/// Sync an org doc to all other projects in the organization
pub(super) async fn sync_org_doc_to_projects(
    org_slug: &str,
    source_project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
) -> Vec<OrgDocSyncResult> {
    let source_path_str = source_project_path.to_string_lossy().to_string();

    let org_projects = match get_org_projects(org_slug, Some(&source_path_str)).await {
        Ok(projects) => projects,
        Err(e) => {
            return vec![OrgDocSyncResult {
                project_path: "<registry>".to_string(),
                success: false,
                error: Some(format!("Failed to get org projects: {e}")),
            }];
        }
    };

    let mut results = Vec::new();

    for project in org_projects {
        let target_path = Path::new(&project.path);
        let result = create_doc_in_project(target_path, slug, title, content, org_slug).await;

        results.push(OrgDocSyncResult {
            project_path: project.path.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        });
    }

    results
}

/// Create a doc in a specific project (used for org doc sync)
/// This is a simpler version that doesn't do org sync recursion
async fn create_doc_in_project(
    project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
    org_slug: &str,
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

    // Skip if already exists (don't overwrite)
    if doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(slug.to_string()));
    }

    let metadata = DocMetadata::new_org_doc(org_slug);
    let doc_content = generate_doc_content(title, content, &metadata);

    fs::write(&doc_path, format_markdown(&doc_content)).await?;

    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(())
}

/// Sync an org doc update to all other projects in the organization
/// Creates the doc if it doesn't exist in a project, or updates it if it does
pub(super) async fn sync_org_doc_update_to_projects(
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
            }];
        }
    };

    let mut results = Vec::new();

    for project in org_projects {
        let target_path = Path::new(&project.path);
        let result =
            update_or_create_doc_in_project(target_path, slug, title, content, org_slug, old_slug)
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
/// If the doc exists, updates it. If it doesn't exist, creates it.
/// If old_slug is provided (slug rename), deletes the old doc and creates new.
async fn update_or_create_doc_in_project(
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

    // Handle slug rename: delete old file if it exists
    if let Some(old) = old_slug {
        if old != slug {
            let old_doc_path = docs_path.join(format!("{old}.md"));
            if old_doc_path.exists() {
                fs::remove_file(&old_doc_path).await?;
            }
        }
    }

    // Check if doc already exists (to preserve created_at)
    let metadata = if doc_path.exists() {
        // Read existing to preserve created_at
        let existing = read_doc_from_disk(&doc_path, slug).await?;
        DocMetadata {
            created_at: existing.metadata.created_at,
            updated_at: now_iso(),
            deleted_at: None, // Clear any soft-delete on update
            is_org_doc: true,
            org_slug: Some(org_slug.to_string()),
        }
    } else {
        // Check if there's an old slug doc to get created_at from
        let old_created_at = if let Some(old) = old_slug {
            let old_doc_path = docs_path.join(format!("{old}.md"));
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

    let doc_content = generate_doc_content(title, content, &metadata);

    fs::write(&doc_path, format_markdown(&doc_content)).await?;

    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(())
}
