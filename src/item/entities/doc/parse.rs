use crate::utils::now_iso;
use super::types::DocMetadata;
/// Parse frontmatter from a doc's lines into metadata fields.
fn parse_frontmatter(frontmatter: &[&str]) -> (String, String, String, Option<String>, bool, Option<String>) {
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
            if !v.is_empty() { deleted_at = Some(v); }
        } else if let Some(value) = line.strip_prefix("isOrgDoc:") {
            is_org_doc = value.trim() == "true";
        } else if let Some(value) = line.strip_prefix("orgSlug:") {
            let v = value.trim().trim_matches('"').to_string();
            if !v.is_empty() { org_slug = Some(v); }
        }
    }
    (title, created_at, updated_at, deleted_at, is_org_doc, org_slug)
}
/// Parse doc content with frontmatter present.
fn parse_with_frontmatter(lines: &[&str], end_idx: usize) -> (String, String, DocMetadata) {
    let frontmatter: Vec<&str> = lines.get(1..=end_idx).unwrap_or(&[]).to_vec();
    let body_start = end_idx.saturating_add(2);
    let (title, created_at, updated_at, deleted_at, is_org_doc, org_slug) =
        parse_frontmatter(&frontmatter);
    let body_lines: Vec<&str> = lines
        .get(body_start..)
        .unwrap_or(&[])
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect();
    let body = if body_lines.first().is_some_and(|l| l.starts_with("# ")) {
        body_lines.get(1..).unwrap_or(&[]).iter()
            .skip_while(|line| line.is_empty())
            .copied().collect::<Vec<_>>().join("\n")
    } else {
        body_lines.join("\n")
    };
    let metadata = DocMetadata {
        created_at: if created_at.is_empty() { now_iso() } else { created_at },
        updated_at: if updated_at.is_empty() { now_iso() } else { updated_at },
        deleted_at,
        is_org_doc,
        org_slug,
    };
    (title, body.trim_end().to_string(), metadata)
}
/// Parse doc content without frontmatter (extract title from first # heading).
fn parse_without_frontmatter(lines: &[&str]) -> (String, String, DocMetadata) {
    let mut title = String::new();
    let mut body_start = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# ") {
            title = line.strip_prefix("# ").unwrap_or("").to_string();
            body_start = i.saturating_add(1);
            break;
        }
    }
    let body = lines.get(body_start..).unwrap_or(&[]).iter()
        .skip_while(|line| line.is_empty())
        .copied().collect::<Vec<_>>().join("\n")
        .trim_end().to_string();
    (title, body, DocMetadata {
        created_at: now_iso(), updated_at: now_iso(),
        deleted_at: None, is_org_doc: false, org_slug: None,
    })
}
/// Parse doc content extracting title, body, and metadata from frontmatter.
pub fn parse_doc_content(content: &str) -> (String, String, DocMetadata) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first() == Some(&"---") {
        if let Some(end_idx) = lines.iter().skip(1).position(|&line| line == "---") {
            return parse_with_frontmatter(&lines, end_idx);
        }
    }
    parse_without_frontmatter(&lines)
}
