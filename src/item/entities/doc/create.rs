use std::path::Path;

use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::registry::get_project_info;
use crate::template::{DocTemplateContext, TemplateEngine};
use crate::utils::{format_markdown, get_centy_path};
use tokio::fs;

use super::error::DocError;
use super::metadata::DocMetadata;
use super::org_sync::sync_org_doc_to_projects;
use super::parse::generate_doc_content;
use super::slug::{slugify, validate_slug};
use super::types::{CreateDocOptions, CreateDocResult};

/// Create a new doc
pub async fn create_doc(
    project_path: &Path,
    options: CreateDocOptions,
) -> Result<CreateDocResult, DocError> {
    // Validate title
    if options.title.trim().is_empty() {
        return Err(DocError::TitleRequired);
    }

    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");

    // Ensure docs directory exists
    if !docs_path.exists() {
        fs::create_dir_all(&docs_path).await?;
    }

    // Generate or validate slug
    let slug = match options.slug {
        Some(s) if !s.trim().is_empty() => {
            let slug = slugify(&s);
            validate_slug(&slug)?;
            slug
        }
        _ => slugify(&options.title),
    };

    // Check if slug already exists
    let doc_path = docs_path.join(format!("{slug}.md"));
    if doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(slug));
    }

    // Get organization info if this is an org doc
    let org_slug = if options.is_org_doc {
        // Get project's organization
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

    // Create metadata (with or without org info)
    let metadata = if let Some(ref org) = org_slug {
        DocMetadata::new_org_doc(org)
    } else {
        DocMetadata::new()
    };

    // Generate doc content with frontmatter
    let doc_content = if let Some(ref template_name) = options.template {
        // Use template engine
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
        // Use default format
        generate_doc_content(&options.title, &options.content, &metadata)
    };

    // Write the doc file (formatted)
    fs::write(&doc_path, format_markdown(&doc_content)).await?;

    // Update manifest timestamp
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let created_file = format!(".centy/docs/{slug}.md");

    // Sync to other org projects if this is an org doc
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
