use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{format_markdown, get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;

use super::content::{generate_doc_content, parse_doc_content, read_doc_from_disk};
use super::error::DocError;
use super::helpers::{slugify, validate_slug};
use super::types::{MoveDocOptions, MoveDocResult};

/// Move a doc to another project
#[allow(unknown_lints, max_lines_per_function)]
pub async fn move_doc(options: MoveDocOptions) -> Result<MoveDocResult, DocError> {
    if options.source_project_path == options.target_project_path {
        return Err(DocError::SameProjectMove);
    }

    let mut source_manifest = read_manifest(&options.source_project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let mut target_manifest = read_manifest(&options.target_project_path)
        .await?
        .ok_or(DocError::TargetNotInitialized)?;

    let source_centy = get_centy_path(&options.source_project_path);
    let source_doc_path = source_centy
        .join("docs")
        .join(format!("{}.md", options.slug));

    if !source_doc_path.exists() {
        return Err(DocError::DocNotFound(options.slug.clone()));
    }

    let _source_doc = read_doc_from_disk(&source_doc_path, &options.slug).await?;

    let target_slug = match options.new_slug {
        Some(ref s) if !s.trim().is_empty() => {
            let slug = slugify(s);
            validate_slug(&slug)?;
            slug
        }
        _ => options.slug.clone(),
    };

    let target_centy = get_centy_path(&options.target_project_path);
    let target_docs_path = target_centy.join("docs");
    fs::create_dir_all(&target_docs_path).await?;
    let target_doc_path = target_docs_path.join(format!("{target_slug}.md"));

    if target_doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(target_slug));
    }

    fs::copy(&source_doc_path, &target_doc_path).await?;

    if target_slug != options.slug {
        let content = fs::read_to_string(&target_doc_path).await?;
        let (title, body, mut metadata) = parse_doc_content(&content);
        metadata.updated_at = now_iso();
        let new_content = format_markdown(&generate_doc_content(&title, &body, &metadata));
        fs::write(&target_doc_path, new_content).await?;
    }

    fs::remove_file(&source_doc_path).await?;

    update_manifest(&mut source_manifest);
    update_manifest(&mut target_manifest);
    write_manifest(&options.source_project_path, &source_manifest).await?;
    write_manifest(&options.target_project_path, &target_manifest).await?;

    let moved_doc = read_doc_from_disk(&target_doc_path, &target_slug).await?;

    Ok(MoveDocResult {
        doc: moved_doc,
        old_slug: options.slug,
        source_manifest,
        target_manifest,
    })
}
