use std::path::Path;

use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{get_centy_path, now_iso};
use tokio::fs;

use super::error::DocError;
use super::metadata::DocMetadata;
use super::org_sync::sync_org_doc_update_to_projects;
use super::parse::generate_doc_content;
use super::read::read_doc_from_disk;
use super::slug::{slugify, validate_slug};
use super::types::{Doc, UpdateDocOptions, UpdateDocResult};

/// Update an existing doc
pub async fn update_doc(
    project_path: &Path,
    slug: &str,
    options: UpdateDocOptions,
) -> Result<UpdateDocResult, DocError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");
    let doc_path = docs_path.join(format!("{slug}.md"));

    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }

    // Read current doc
    let current = read_doc_from_disk(&doc_path, slug).await?;

    // Apply updates
    let new_title = options.title.unwrap_or(current.title);
    let new_content = options.content.unwrap_or(current.content);

    // Handle slug rename
    let new_slug = match options.new_slug {
        Some(s) if !s.trim().is_empty() && s != slug => {
            let new_slug = slugify(&s);
            validate_slug(&new_slug)?;

            // Check if new slug already exists
            let new_path = docs_path.join(format!("{new_slug}.md"));
            if new_path.exists() {
                return Err(DocError::SlugAlreadyExists(new_slug));
            }

            Some(new_slug)
        }
        _ => None,
    };

    // Create updated metadata (preserve org doc fields)
    let updated_metadata = DocMetadata {
        created_at: current.metadata.created_at.clone(),
        updated_at: now_iso(),
        deleted_at: current.metadata.deleted_at.clone(),
        is_org_doc: current.metadata.is_org_doc,
        org_slug: current.metadata.org_slug.clone(),
    };

    // Generate updated content
    let doc_content = generate_doc_content(&new_title, &new_content, &updated_metadata);

    // Handle file rename or update
    let final_slug = if let Some(ref new_slug) = new_slug {
        // Remove old file
        fs::remove_file(&doc_path).await?;

        // Write new file
        let new_path = docs_path.join(format!("{new_slug}.md"));
        fs::write(&new_path, &doc_content).await?;

        new_slug.clone()
    } else {
        // Just update the existing file
        fs::write(&doc_path, &doc_content).await?;

        slug.to_string()
    };

    // Update manifest timestamp
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let doc = Doc {
        slug: final_slug.clone(),
        title: new_title.clone(),
        content: new_content.clone(),
        metadata: updated_metadata,
    };

    // Sync to other org projects if this is an org doc
    let sync_results = if doc.metadata.is_org_doc {
        if let Some(ref org) = doc.metadata.org_slug {
            // Determine the old slug for rename handling
            let old_slug_for_sync = new_slug.as_ref().map(|_| slug);
            sync_org_doc_update_to_projects(
                org,
                project_path,
                &final_slug,
                &new_title,
                &new_content,
                old_slug_for_sync,
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
