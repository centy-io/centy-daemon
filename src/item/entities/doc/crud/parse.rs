use crate::utils::now_iso;

use super::parse_helpers::parse_with_frontmatter;
use super::types::DocMetadata;

/// Parse doc content extracting title, body, and metadata from frontmatter
pub fn parse_doc_content(content: &str) -> (String, String, DocMetadata) {
    let lines: Vec<&str> = content.lines().collect();

    if lines.first() == Some(&"---") {
        if let Some(result) = parse_with_frontmatter(&lines) {
            return result;
        }
    }

    // No frontmatter - extract title from first # heading
    let mut title = String::new();
    let mut body_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# ") {
            title = line.strip_prefix("# ").unwrap_or("").to_string();
            body_start = i.saturating_add(1);
            break;
        }
    }

    let body = lines
        .get(body_start..)
        .unwrap_or(&[])
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();

    (
        title,
        body,
        DocMetadata {
            created_at: now_iso(),
            updated_at: now_iso(),
            deleted_at: None,
            is_org_doc: false,
            org_slug: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doc_content_with_frontmatter() {
        let content = "---\ntitle: \"My Doc\"\ncreatedAt: \"2024-01-01T00:00:00Z\"\nupdatedAt: \"2024-01-02T00:00:00Z\"\n---\n\n# My Doc\n\nThis is the content.";
        let (title, body, metadata) = parse_doc_content(content);
        assert_eq!(title, "My Doc");
        assert_eq!(body, "This is the content.");
        assert_eq!(metadata.created_at, "2024-01-01T00:00:00Z");
        assert_eq!(metadata.updated_at, "2024-01-02T00:00:00Z");
    }

    #[test]
    fn test_parse_doc_content_without_frontmatter() {
        let content = "# Simple Doc\n\nJust some content here.";
        let (title, body, _metadata) = parse_doc_content(content);
        assert_eq!(title, "Simple Doc");
        assert_eq!(body, "Just some content here.");
    }
}
