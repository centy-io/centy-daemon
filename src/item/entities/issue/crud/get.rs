use super::super::id::{is_valid_issue_file, is_valid_issue_folder};
use super::super::metadata::{IssueFrontmatter, IssueMetadata};
use super::super::reconcile::reconcile_display_numbers;
use super::migrate::migrate_issue_to_new_format;
use super::read::read_issue_from_frontmatter;
use super::types::{Issue, IssueCrudError};
use crate::manifest::read_manifest;
use crate::utils::{get_centy_path, strip_centy_md_header};
use mdstore::parse_frontmatter;
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

async fn match_entry_by_display_number(
    entry: &fs::DirEntry,
    display_number: u32,
    issues_path: &Path,
) -> Result<Option<Issue>, IssueCrudError> {
    let file_type = entry.file_type().await?;
    let name = entry.file_name();
    let Some(name) = name.to_str() else {
        return Ok(None);
    };
    if file_type.is_file() && is_valid_issue_file(name) {
        return match_file_entry(entry, name, display_number).await;
    }
    if file_type.is_dir() && is_valid_issue_folder(name) {
        return match_dir_entry(entry, name, display_number, issues_path).await;
    }
    Ok(None)
}

async fn match_file_entry(
    entry: &fs::DirEntry,
    name: &str,
    display_number: u32,
) -> Result<Option<Issue>, IssueCrudError> {
    let Ok(content) = fs::read_to_string(entry.path()).await else {
        return Ok(None);
    };
    let Ok((fm, _, _)) = parse_frontmatter::<IssueFrontmatter>(strip_centy_md_header(&content))
    else {
        return Ok(None);
    };
    if fm.display_number != display_number {
        return Ok(None);
    }
    let issue_id = name.trim_end_matches(".md");
    Ok(Some(
        read_issue_from_frontmatter(&entry.path(), issue_id).await?,
    ))
}

async fn match_dir_entry(
    entry: &fs::DirEntry,
    name: &str,
    display_number: u32,
    issues_path: &Path,
) -> Result<Option<Issue>, IssueCrudError> {
    let metadata_path = entry.path().join("metadata.json");
    if !metadata_path.exists() {
        return Ok(None);
    }
    let Ok(content) = fs::read_to_string(&metadata_path).await else {
        return Ok(None);
    };
    let Ok(meta) = serde_json::from_str::<IssueMetadata>(&content) else {
        return Ok(None);
    };
    if meta.common.display_number != display_number {
        return Ok(None);
    }
    Ok(Some(
        migrate_issue_to_new_format(issues_path, &entry.path(), name).await?,
    ))
}
