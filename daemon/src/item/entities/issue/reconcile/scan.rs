use super::super::id::is_valid_issue_file;
use super::super::metadata::IssueFrontmatter;
use super::types::{IssueInfo, ReconcileError};
use crate::utils::strip_centy_md_header;
use mdstore::parse_frontmatter;
use std::path::Path;
use tokio::fs;
/// Scan the issues directory and collect issue info.
pub async fn scan_issues(issues_path: &Path) -> Result<Vec<IssueInfo>, ReconcileError> {
    let mut issues: Vec<IssueInfo> = Vec::new();
    let mut entries = fs::read_dir(issues_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        if file_type.is_dir() || !is_valid_issue_file(&name) {
            continue;
        }
        let Ok(content) = fs::read_to_string(entry.path()).await else {
            continue;
        };
        let frontmatter: IssueFrontmatter =
            match parse_frontmatter::<IssueFrontmatter>(strip_centy_md_header(&content)) {
                Ok((fm, _, _)) => fm,
                Err(_) => continue,
            };
        let issue_id = name.trim_end_matches(".md").to_string();
        issues.push(IssueInfo {
            id: issue_id,
            display_number: frontmatter.display_number,
            created_at: frontmatter.created_at,
        });
    }
    Ok(issues)
}
/// Get the next available display number.
///
/// Scans all existing issues and returns max + 1.
pub async fn get_next_display_number(issues_path: &Path) -> Result<u32, ReconcileError> {
    if !issues_path.exists() {
        return Ok(1);
    }
    let mut max_number: u32 = 0;
    let mut entries = fs::read_dir(issues_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        if file_type.is_dir() || !is_valid_issue_file(&name) {
            continue;
        }
        if let Ok(content) = fs::read_to_string(entry.path()).await {
            if let Ok((frontmatter, _, _)) =
                parse_frontmatter::<IssueFrontmatter>(strip_centy_md_header(&content))
            {
                max_number = max_number.max(frontmatter.display_number);
            }
        }
    }
    Ok(max_number.saturating_add(1))
}
