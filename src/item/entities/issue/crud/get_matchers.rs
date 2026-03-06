use super::super::id::{is_valid_issue_file, is_valid_issue_folder};
use super::super::metadata::{IssueFrontmatter, IssueMetadata};
use super::migrate::migrate_issue_to_new_format;
use super::read::read_issue_from_frontmatter;
use super::types::{Issue, IssueCrudError};
use crate::utils::strip_centy_md_header;
use mdstore::parse_frontmatter;
use std::path::Path;
use tokio::fs;

pub(super) async fn match_entry_by_display_number(
    entry: &fs::DirEntry,
    display_number: u32,
    issues_path: &Path,
) -> Result<Option<Issue>, IssueCrudError> {
    let file_type = entry.file_type().await?;
    let file_name_os = entry.file_name();
    let Some(name) = file_name_os.to_str() else {
        return Ok(None);
    };
    if !file_type.is_dir() && is_valid_issue_file(name) {
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
