use super::super::reconcile::reconcile_display_numbers;
use super::get_matchers::match_entry_by_display_number;
use super::migrate::migrate_issue_to_new_format;
use super::read::read_issue_from_frontmatter;
use super::types::{Issue, IssueCrudError};
use crate::manifest::read_manifest;
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;

pub async fn get_issue(project_path: &Path, issue_number: &str) -> Result<Issue, IssueCrudError> {
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
        if let Some(issue) =
            match_entry_by_display_number(&entry, display_number, &issues_path).await?
        {
            return Ok(issue);
        }
    }
    Err(IssueCrudError::IssueDisplayNumberNotFound(display_number))
}
