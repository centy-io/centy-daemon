use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{get_centy_path, now_iso};
use tokio::fs;

use super::error::DocError;
use super::parse::{generate_doc_content, parse_doc_content};
use super::read::read_doc_from_disk;
use super::slug::{slugify, validate_slug};
use super::types::{MoveDocOptions, MoveDocResult};

/// Move a doc to another project
///
/// The doc is transferred to the target project and deleted from the source.
/// A new slug can be provided if there's a conflict in the target project.
///
/// # Arguments
/// * `options` - Move options specifying source, target, slug, and optional new slug
///
/// # Returns
/// The moved doc with the original slug for reference, plus both manifests
pub async fn move_doc(options: MoveDocOptions) -> Result<MoveDocResult, DocError> {
    // Verify not same project
    if options.source_project_path == options.target_project_path {
        return Err(DocError::SameProjectMove);
    }

    // Validate source project is initialized
    let mut source_manifest = read_manifest(&options.source_project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    // Validate target project is initialized
    let mut target_manifest = read_manifest(&options.target_project_path)
        .await?
        .ok_or(DocError::TargetNotInitialized)?;

    // Read source doc
    let source_centy = get_centy_path(&options.source_project_path);
    let source_doc_path = source_centy
        .join("docs")
        .join(format!("{}.md", options.slug));

    if !source_doc_path.exists() {
        return Err(DocError::DocNotFound(options.slug.clone()));
    }

    // Validate source doc is readable
    let _source_doc = read_doc_from_disk(&source_doc_path, &options.slug).await?;

    // Determine target slug
    let target_slug = match options.new_slug {
        Some(ref s) if !s.trim().is_empty() => {
            let slug = slugify(s);
            validate_slug(&slug)?;
            slug
        }
        _ => options.slug.clone(),
    };

    // Check for conflict in target project
    let target_centy = get_centy_path(&options.target_project_path);
    let target_docs_path = target_centy.join("docs");
    fs::create_dir_all(&target_docs_path).await?;
    let target_doc_path = target_docs_path.join(format!("{target_slug}.md"));

    if target_doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(target_slug));
    }

    // Copy file to target (preserving metadata)
    fs::copy(&source_doc_path, &target_doc_path).await?;

    // If slug changed, update the content to reflect new title header
    if target_slug != options.slug {
        // Re-read and re-write with potentially updated metadata
        let content = fs::read_to_string(&target_doc_path).await?;
        let (title, body, mut metadata) = parse_doc_content(&content);
        metadata.updated_at = now_iso();
        let new_content = generate_doc_content(&title, &body, &metadata);
        fs::write(&target_doc_path, new_content).await?;
    }

    // Delete from source project
    fs::remove_file(&source_doc_path).await?;

    // Update both manifests
    update_manifest(&mut source_manifest);
    update_manifest(&mut target_manifest);
    write_manifest(&options.source_project_path, &source_manifest).await?;
    write_manifest(&options.target_project_path, &target_manifest).await?;

    // Read the moved doc
    let moved_doc = read_doc_from_disk(&target_doc_path, &target_slug).await?;

    Ok(MoveDocResult {
        doc: moved_doc,
        old_slug: options.slug,
        source_manifest,
        target_manifest,
    })
}
