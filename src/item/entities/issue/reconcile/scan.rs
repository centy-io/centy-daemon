#![allow(unknown_lints, max_nesting_depth)]
use super::super::id::{is_valid_issue_file, is_valid_issue_folder};
use super::super::metadata::{IssueFrontmatter, IssueMetadata};
use super::types::{IssueInfo, ReconcileError};
use mdstore::parse_frontmatter;
use std::path::Path;
use tokio::fs;
/// Scan the issues directory and collect issue info from both formats.
pub async fn scan_issues(issues_path: &Path) -> Result<Vec<IssueInfo>, ReconcileError> {
    let mut issues: Vec<IssueInfo> = Vec::new();
    let mut entries = fs::read_dir(issues_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        if file_type.is_file() && is_valid_issue_file(&name) {
            let content = match fs::read_to_string(entry.path()).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            let frontmatter: IssueFrontmatter =
                match parse_frontmatter::<IssueFrontmatter>(&content) {
                    Ok((fm, _, _)) => fm,
                    Err(_) => continue,
                };
            let issue_id = name.trim_end_matches(".md").to_string();
            issues.push(IssueInfo {
                id: issue_id,
                is_new_format: true,
                display_number: frontmatter.display_number,
                created_at: frontmatter.created_at,
            });
        } else if file_type.is_dir() && is_valid_issue_folder(&name) {
            let metadata_path = entry.path().join("metadata.json");
            if !metadata_path.exists() {
                continue;
            }
            let content = match fs::read_to_string(&metadata_path).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            let metadata: IssueMetadata = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue,
            };
            issues.push(IssueInfo {
                id: name,
                is_new_format: false,
                display_number: metadata.common.display_number,
                created_at: metadata.common.created_at,
            });
        }
    }
    Ok(issues)
}
/// Get the next available display number.
///
/// Scans all existing issues (both formats) and returns max + 1.
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
        if file_type.is_file() && is_valid_issue_file(&name) {
            if let Ok(content) = fs::read_to_string(entry.path()).await {
                if let Ok((frontmatter, _, _)) = parse_frontmatter::<IssueFrontmatter>(&content) {
                    max_number = max_number.max(frontmatter.display_number);
                }
            }
        } else if file_type.is_dir() && is_valid_issue_folder(&name) {
            let metadata_path = entry.path().join("metadata.json");
            if !metadata_path.exists() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&metadata_path).await {
                if let Ok(metadata) = serde_json::from_str::<IssueMetadata>(&content) {
                    max_number = max_number.max(metadata.common.display_number);
                }
            }
        }
    }
    Ok(max_number.saturating_add(1))
}
