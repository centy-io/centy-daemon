use super::types::DocMetadata;

/// Escape special characters in YAML strings
pub fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_yaml_string() {
        assert_eq!(escape_yaml_string("simple"), "simple");
        assert_eq!(escape_yaml_string("with \"quotes\""), "with \\\"quotes\\\"");
        assert_eq!(escape_yaml_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_generate_doc_content() {
        let metadata = DocMetadata {
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
            deleted_at: None,
            is_org_doc: false,
            org_slug: None,
        };
        let content = generate_doc_content("Test Title", "Body text", &metadata);

        assert!(content.contains("title: \"Test Title\""));
        assert!(content.contains("# Test Title"));
        assert!(content.contains("Body text"));
    }
}
