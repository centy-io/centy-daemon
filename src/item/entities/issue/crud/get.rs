#![allow(unknown_lints, max_nesting_depth)]
use super::migrate::migrate_issue_to_new_format;
use super::read::{read_issue_from_frontmatter, read_issue_from_legacy_folder};
use super::types::{Issue, IssueCrudError};
use super::super::id::{is_valid_issue_file, is_valid_issue_folder};
use super::super::metadata::{IssueFrontmatter, IssueMetadata};
use super::super::reconcile::reconcile_display_numbers;
use crate::manifest::read_manifest;
use crate::utils::get_centy_path;
use mdstore::parse_frontmatter;
use std::path::Path;
use tokio::fs;

pub async fn get_issue(
    project_path: &Path,
    issue_number: &str,
) -> Result<Issue, IssueCrudError> {
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    let issue_file_path = issues_path.join(format!("{issue_number}.md"));
    if issue_file_path.exists() {
        return read_issue_from_frontmatter(&issue_file_path, issue_number).await;
    }
    let issue_folder_path = issues_path.join(issue_number);
    if issue_folder_path.exists() {
        return migrate_issue_to_new_format(&issues_path, &issue_folder_path, issue_number).await;
    }
    Err(IssueCrudError::IssueNotFound(issue_number.to_string()))
}

pub async fn get_issue_by_display_number(
    project_path: &Path,
    display_number: u32,
) -> Result<Issue, IssueCrudError> {
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    if !issues_path.exists() {
        return Err(IssueCrudError::IssueDisplayNumberNotFound(display_number));
    }
    reconcile_display_numbers(&issues_path).await?;
    let mut entries = fs::read_dir(&issues_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        if let Some(name) = entry.file_name().to_str() {
            if file_type.is_file() && is_valid_issue_file(name) {
                if let Ok(content) = fs::read_to_string(entry.path()).await {
                    if let Ok((fm, _, _)) = parse_frontmatter::<IssueFrontmatter>(&content) {
                        if fm.display_number == display_number {
                            let issue_id = name.trim_end_matches(".md");
                            return read_issue_from_frontmatter(&entry.path(), issue_id).await;
                        }
                    }
                }
            } else if file_type.is_dir() && is_valid_issue_folder(name) {
                let metadata_path = entry.path().join("metadata.json");
                if !metadata_path.exists() { continue; }
                if let Ok(content) = fs::read_to_string(&metadata_path).await {
                    if let Ok(meta) = serde_json::from_str::<IssueMetadata>(&content) {
                        if meta.common.display_number == display_number {
                            return migrate_issue_to_new_format(&issues_path, &entry.path(), name).await;
                        }
                    }
                }
            }
        }
    }
    Err(IssueCrudError::IssueDisplayNumberNotFound(display_number))
}
