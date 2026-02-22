use std::path::Path;

use tokio::fs;

use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::registry::get_org_projects;
use crate::utils::{format_markdown, get_centy_path};

use super::format::generate_doc_content;
use super::options::OrgDocSyncResult;
use super::types::{DocError, DocMetadata};

/// Create a doc in a specific project (used for org doc sync, no recursion)
pub async fn create_doc_in_project(
    project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
    org_slug: &str,
) -> Result<(), DocError> {
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;
    let docs_path = get_centy_path(project_path).join("docs");
    if !docs_path.exists() {
        fs::create_dir_all(&docs_path).await?;
    }
    let doc_path = docs_path.join(format!("{slug}.md"));
    if doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(slug.to_string()));
    }
    let metadata = DocMetadata::new_org_doc(org_slug);
    fs::write(&doc_path, format_markdown(&generate_doc_content(title, content, &metadata))).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    Ok(())
}

/// Sync an org doc to all other projects in the organization
pub async fn sync_org_doc_to_projects(
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
