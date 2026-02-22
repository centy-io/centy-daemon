use std::path::Path;

use tokio::fs;

use super::parse::parse_doc_content;
use super::types::{Doc, DocError};

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
