use std::path::Path;

use tokio::fs;

use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::registry::get_org_projects;
use crate::utils::{format_markdown, get_centy_path, now_iso};

use super::format::generate_doc_content;
use super::io::read_doc_from_disk;
use super::options::OrgDocSyncResult;
use super::types::{DocError, DocMetadata};

async fn resolve_created_at(docs_path: &std::path::Path, _slug: &str, old_slug: Option<&str>) -> String {
    if let Some(old) = old_slug {
        let old_path = docs_path.join(format!("{old}.md"));
        if old_path.exists() {
            if let Ok(d) = read_doc_from_disk(&old_path, old).await {
                return d.metadata.created_at;
            }
        }
    }
    now_iso()
}

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
    let docs_path = get_centy_path(project_path).join("docs");
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
    let created_at = if doc_path.exists() {
        read_doc_from_disk(&doc_path, slug).await?.metadata.created_at
    } else {
        resolve_created_at(&docs_path, slug, old_slug).await
    };
    let metadata = DocMetadata {
        created_at,
        updated_at: now_iso(),
        deleted_at: None,
        is_org_doc: true,
        org_slug: Some(org_slug.to_string()),
    };
    fs::write(&doc_path, format_markdown(&generate_doc_content(title, content, &metadata))).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    Ok(())
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
            }];
        }
    };
    let mut results = Vec::new();
    for project in org_projects {
        let r = update_or_create_doc_in_project(
            Path::new(&project.path), slug, title, content, org_slug, old_slug,
        ).await;
        results.push(OrgDocSyncResult {
            project_path: project.path.clone(),
            success: r.is_ok(),
            error: r.err().map(|e| e.to_string()),
        });
    }
    results
}
