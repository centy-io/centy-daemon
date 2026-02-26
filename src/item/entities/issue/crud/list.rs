#![allow(unknown_lints, max_nesting_depth)]
use super::get::get_issue;
use super::migrate::migrate_issue_to_new_format;
use super::read::read_issue_from_frontmatter;
use super::types::{GetIssuesByUuidResult, Issue, IssueCrudError, IssueWithProject};
use super::super::id::{is_uuid, is_valid_issue_file, is_valid_issue_folder};
use super::super::reconcile::reconcile_display_numbers;
use crate::manifest::read_manifest;
use crate::registry::ProjectInfo;
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;

#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn list_issues(
    project_path: &Path,
    status_filter: Option<&str>,
    priority_filter: Option<u32>,
    draft_filter: Option<bool>,
    include_deleted: bool,
) -> Result<Vec<Issue>, IssueCrudError> {
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    if !issues_path.exists() { return Ok(Vec::new()); }
    reconcile_display_numbers(&issues_path).await?;
    let mut issues = Vec::new();
    let mut entries = fs::read_dir(&issues_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        if let Some(name) = entry.file_name().to_str() {
            let read_result = if file_type.is_file() && is_valid_issue_file(name) {
                let issue_id = name.trim_end_matches(".md");
                read_issue_from_frontmatter(&entry.path(), issue_id).await
            } else if file_type.is_dir() && is_valid_issue_folder(name) {
                migrate_issue_to_new_format(&issues_path, &entry.path(), name).await
            } else {
                continue;
            };
            if let Ok(issue) = read_result {
                let status_match = status_filter.is_none_or(|s| issue.metadata.status == s);
                let priority_match = priority_filter.is_none_or(|p| issue.metadata.priority == p);
                let draft_match = draft_filter.is_none_or(|d| issue.metadata.draft == d);
                let deleted_match = include_deleted || issue.metadata.deleted_at.is_none();
                if status_match && priority_match && draft_match && deleted_match {
                    issues.push(issue);
                }
            }
        }
    }
    issues.sort_by_key(|i| i.metadata.display_number);
    Ok(issues)
}

pub async fn get_issues_by_uuid(
    uuid: &str,
    projects: &[ProjectInfo],
) -> Result<GetIssuesByUuidResult, IssueCrudError> {
    if !is_uuid(uuid) {
        return Err(IssueCrudError::InvalidIssueFormat(
            "Only UUID format is supported for global search".to_string(),
        ));
    }
    let mut found_issues = Vec::new();
    let mut errors = Vec::new();
    for project in projects {
        if !project.initialized { continue; }
        let project_path = Path::new(&project.path);
        match get_issue(project_path, uuid).await {
            Ok(issue) => {
                let project_name = project.name.clone().unwrap_or_else(|| {
                    project_path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| project.path.clone())
                });
                found_issues.push(IssueWithProject { issue, project_path: project.path.clone(), project_name });
            }
            Err(IssueCrudError::IssueNotFound(_) | IssueCrudError::NotInitialized) => {}
            Err(e) => { errors.push(format!("Error searching {}: {}", project.path, e)); }
        }
    }
    Ok(GetIssuesByUuidResult { issues: found_issues, errors })
}
