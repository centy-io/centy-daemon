use crate::utils::now_iso;

use super::types::DocMetadata;

/// Parse YAML frontmatter fields from lines between the --- delimiters
pub fn parse_frontmatter_fields(
    frontmatter: &[&str],
) -> (String, String, String, Option<String>, bool, Option<String>) {
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

    (title, created_at, updated_at, deleted_at, is_org_doc, org_slug)
}

/// Extract body text from lines, skipping the leading title heading if present
pub fn extract_body(lines: &[&str]) -> String {
    let body_lines: Vec<&str> = lines
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect();

    if body_lines.first().is_some_and(|l| l.starts_with("# ")) {
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
    }
}

/// Try to parse content that has YAML frontmatter (---...---)
pub fn parse_with_frontmatter(lines: &[&str]) -> Option<(String, String, DocMetadata)> {
    let end_idx = lines.iter().skip(1).position(|&line| line == "---")?;
    let frontmatter = lines.get(1..=end_idx).unwrap_or(&[]).to_vec();
    let body_start = end_idx.saturating_add(2);

    let (title, created_at, updated_at, deleted_at, is_org_doc, org_slug) =
        parse_frontmatter_fields(&frontmatter);

    let body = extract_body(lines.get(body_start..).unwrap_or(&[]));

    let metadata = DocMetadata {
        created_at: if created_at.is_empty() { now_iso() } else { created_at },
        updated_at: if updated_at.is_empty() { now_iso() } else { updated_at },
        deleted_at,
        is_org_doc,
        org_slug,
    };

    Some((title, body.trim_end().to_string(), metadata))
}
