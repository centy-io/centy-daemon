use super::metadata::DocMetadata;
use crate::utils::now_iso;

pub(super) fn generate_doc_content(title: &str, content: &str, metadata: &DocMetadata) -> String {
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

pub(super) fn parse_doc_content(content: &str) -> (String, String, DocMetadata) {
    let lines: Vec<&str> = content.lines().collect();

    if lines.first() == Some(&"---") {
        if let Some(end_idx) = lines.iter().skip(1).position(|&line| line == "---") {
            let frontmatter: Vec<&str> = lines.get(1..=end_idx).unwrap_or(&[]).to_vec();
            let body_start = end_idx.saturating_add(2);

            let mut title = String::new();
            let mut created_at = String::new();
            let mut updated_at = String::new();
            let mut deleted_at: Option<String> = None;
            let mut is_org_doc = false;
            let mut org_slug: Option<String> = None;

            for line in frontmatter {
                if let Some(value) = line.strip_prefix("title:") {
                    title = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("createdAt:") {
                    created_at = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("updatedAt:") {
                    updated_at = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("deletedAt:") {
                    let v = value.trim().trim_matches('"').to_string();
                    if !v.is_empty() {
                        deleted_at = Some(v);
                    }
                } else if let Some(value) = line.strip_prefix("isOrgDoc:") {
                    is_org_doc = value.trim() == "true";
                } else if let Some(value) = line.strip_prefix("orgSlug:") {
                    let v = value.trim().trim_matches('"').to_string();
                    if !v.is_empty() {
                        org_slug = Some(v);
                    }
                }
            }

            let body_lines: Vec<&str> = lines
                .get(body_start..)
                .unwrap_or(&[])
                .iter()
                .skip_while(|line| line.is_empty())
                .copied()
                .collect();

            let body = if body_lines.first().is_some_and(|l| l.starts_with("# ")) {
                body_lines
                    .get(1..)
                    .unwrap_or(&[])
                    .iter()
                    .skip_while(|line| line.is_empty())
                    .copied()
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                body_lines.join("\n")
            };

            let metadata = DocMetadata {
                created_at: if created_at.is_empty() {
                    now_iso()
                } else {
                    created_at
                },
                updated_at: if updated_at.is_empty() {
                    now_iso()
                } else {
                    updated_at
                },
                deleted_at,
                is_org_doc,
                org_slug,
            };

            return (title, body.trim_end().to_string(), metadata);
        }
    }

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

fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doc_content_with_frontmatter() {
        let content = r#"---
title: "My Doc"
createdAt: "2024-01-01T00:00:00Z"
updatedAt: "2024-01-02T00:00:00Z"
---

# My Doc

This is the content."#;

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

    #[test]
    fn test_escape_yaml_string() {
        assert_eq!(escape_yaml_string("simple"), "simple");
        assert_eq!(escape_yaml_string("with \"quotes\""), "with \\\"quotes\\\"");
        assert_eq!(escape_yaml_string("back\\slash"), "back\\\\slash");
    }
}
