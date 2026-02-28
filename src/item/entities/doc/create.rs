use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::registry::{get_org_projects, get_project_info};
use crate::template::{DocTemplateContext, TemplateEngine};
use crate::utils::{format_markdown, get_centy_path};
use std::path::Path;
use tokio::fs;

use super::content::{generate_doc_content, read_doc_from_disk};
use super::error::DocError;
use super::helpers::{slugify, validate_slug};
use super::types::{CreateDocOptions, CreateDocResult, DocMetadata, OrgDocSyncResult};

/// Create a new doc
pub async fn create_doc(
    project_path: &Path,
    options: CreateDocOptions,
) -> Result<CreateDocResult, DocError> {
    if options.title.trim().is_empty() {
        return Err(DocError::TitleRequired);
    }
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");
    if !docs_path.exists() {
        fs::create_dir_all(&docs_path).await?;
    }
    let slug = match options.slug {
        Some(s) if !s.trim().is_empty() => {
            let slug = slugify(&s);
            validate_slug(&slug)?;
            slug
        }
        _ => slugify(&options.title),
    };
    let doc_path = docs_path.join(format!("{slug}.md"));
    if doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(slug));
    }
    let org_slug = if options.is_org_doc {
        let project_path_str = project_path.to_string_lossy().to_string();
        let project_info = get_project_info(&project_path_str)
            .await
            .map_err(|e| DocError::RegistryError(e.to_string()))?;
        match project_info.and_then(|p| p.organization_slug) {
            Some(slug) => Some(slug),
            None => return Err(DocError::NoOrganization),
        }
    } else {
        None
    };
    let metadata = if let Some(ref org) = org_slug {
        DocMetadata::new_org_doc(org)
    } else {
        DocMetadata::new()
    };
    let doc_content = if let Some(ref template_name) = options.template {
        let template_engine = TemplateEngine::new();
        let context = DocTemplateContext {
            title: options.title.clone(),
            content: options.content.clone(),
            slug: slug.clone(),
            created_at: metadata.created_at.clone(),
            updated_at: metadata.updated_at.clone(),
        };
        template_engine
            .render_doc(project_path, template_name, &context)
            .await?
    } else {
        generate_doc_content(&options.title, &options.content, &metadata)
    };
    fs::write(&doc_path, format_markdown(&doc_content)).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    let created_file = format!(".centy/docs/{slug}.md");
    let sync_results = if let Some(ref org) = org_slug {
        sync_org_doc_to_projects(org, project_path, &slug, &options.title, &options.content).await
    } else {
        Vec::new()
    };
    Ok(CreateDocResult {
        slug,
        created_file,
        manifest,
        sync_results,
    })
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
            }]
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
    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");
    if !docs_path.exists() {
        fs::create_dir_all(&docs_path).await?;
    }
    let doc_path = docs_path.join(format!("{slug}.md"));
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
