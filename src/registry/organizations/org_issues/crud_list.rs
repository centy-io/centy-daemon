//! List and lookup operations for org issues.
use super::crud_ops::read_org_issue_file;
use super::crud_types::{ListOrgIssuesOptions, OrgIssue, OrgIssueError};
use super::paths::get_org_issues_dir;
use tokio::fs;
pub async fn get_org_issue_by_display_number(
    org_slug: &str,
    display_number: u32,
) -> Result<OrgIssue, OrgIssueError> {
    let issues_dir = get_org_issues_dir(org_slug)?;
    if !issues_dir.exists() {
        return Err(OrgIssueError::NotFound(display_number.to_string()));
    }
    let mut entries = fs::read_dir(&issues_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if !name.ends_with(".md") {
            continue;
        }
        let issue_id = name.trim_end_matches(".md");
        if let Ok(issue) = read_org_issue_file(&issues_dir, issue_id).await {
            if issue.display_number == display_number {
                return Ok(issue);
            }
        }
    }
    Err(OrgIssueError::NotFound(display_number.to_string()))
}
pub async fn list_org_issues(
    org_slug: &str,
    opts: ListOrgIssuesOptions,
) -> Result<Vec<OrgIssue>, OrgIssueError> {
    let issues_dir = get_org_issues_dir(org_slug)?;
    if !issues_dir.exists() {
        return Ok(Vec::new());
    }
    let mut issues = Vec::new();
    let mut entries = fs::read_dir(&issues_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if !name.ends_with(".md") {
            continue;
        }
        let issue_id = name.trim_end_matches(".md");
        let Ok(issue) = read_org_issue_file(&issues_dir, issue_id).await else {
            continue;
        };
        if let Some(ref status) = opts.status {
            if &issue.status != status {
                continue;
            }
        }
        if let Some(priority) = opts.priority {
            if issue.priority != priority {
                continue;
            }
        }
        if let Some(ref project) = opts.referenced_project {
            if !issue.referenced_projects.contains(project) {
                continue;
            }
        }
        issues.push(issue);
    }
    issues.sort_by_key(|i| i.display_number);
    Ok(issues)
}
