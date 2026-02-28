pub use super::parse::parse_doc_content;
use std::path::Path;
use tokio::fs;

use super::error::DocError;
use super::helpers::escape_yaml_string;
use super::types::{Doc, DocMetadata};

/// Read a doc from disk
pub async fn read_doc_from_disk(doc_path: &Path, slug: &str) -> Result<Doc, DocError> {
    let content = fs::read_to_string(doc_path).await?;
    let (title, body, metadata) = parse_doc_content(&content);
    Ok(Doc {
        slug: slug.to_string(),
        title,
        content: body,
        metadata,
    })
}

/// Generate doc content with YAML frontmatter
pub fn generate_doc_content(title: &str, content: &str, metadata: &DocMetadata) -> String {
    let deleted_line = metadata
        .deleted_at
        .as_ref()
        .map(|d| format!("\ndeletedAt: \"{d}\""))
        .unwrap_or_default();
    let org_doc_line = if metadata.is_org_doc {
        "\nisOrgDoc: true".to_string()
    } else {
        String::new()
    };
    let org_slug_line = metadata
        .org_slug
        .as_ref()
        .map(|s| format!("\norgSlug: \"{s}\""))
        .unwrap_or_default();
    format!(
        "---\ntitle: \"{}\"\ncreatedAt: \"{}\"\nupdatedAt: \"{}\"{}{}{}\n---\n\n# {}\n\n{}",
        escape_yaml_string(title),
        metadata.created_at,
        metadata.updated_at,
        deleted_line,
        org_doc_line,
        org_slug_line,
        title,
        content
    )
}
