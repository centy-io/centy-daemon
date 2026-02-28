//! Create/read/list operations for org issues.
use super::crud_types::{ListOrgIssuesOptions, OrgIssue, OrgIssueError, OrgIssueFrontmatter};
use super::paths::get_org_issues_dir;
use crate::item::entities::issue::org_registry::get_next_org_display_number;
use crate::utils::{now_iso, strip_centy_md_header};
use mdstore::{generate_frontmatter, parse_frontmatter};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;
pub async fn read_org_issue_file(
    issues_dir: &Path,
    issue_id: &str,
) -> Result<OrgIssue, OrgIssueError> {
    let file_path = issues_dir.join(format!("{issue_id}.md"));
    if !file_path.exists() {
        return Err(OrgIssueError::NotFound(issue_id.to_string()));
    }
    let content = fs::read_to_string(&file_path).await?;
    let (frontmatter, title, description) =
        parse_frontmatter::<OrgIssueFrontmatter>(strip_centy_md_header(&content))?;
    Ok(OrgIssue {
        id: issue_id.to_string(),
        display_number: frontmatter.display_number,
        title,
        description,
        status: frontmatter.status,
        priority: frontmatter.priority,
        created_at: frontmatter.created_at,
        updated_at: frontmatter.updated_at,
        custom_fields: frontmatter.custom_fields,
        referenced_projects: frontmatter.referenced_projects,
    })
}
pub async fn create_org_issue(
    org_slug: &str,
    title: &str,
    description: &str,
    priority: u32,
    status: &str,
    custom_fields: impl Into<HashMap<String, String>>,
    referenced_projects: Vec<String>,
) -> Result<OrgIssue, OrgIssueError> {
    let custom_fields = custom_fields.into();
    if title.trim().is_empty() {
        return Err(OrgIssueError::TitleRequired);
    }
    let issues_dir = get_org_issues_dir(org_slug)?;
    fs::create_dir_all(&issues_dir).await?;
    let issue_id = Uuid::new_v4().to_string();
    let display_number = get_next_org_display_number(org_slug).await?;
    let now = now_iso();
    let frontmatter = OrgIssueFrontmatter {
        display_number,
        status: status.to_string(),
        priority,
        created_at: now.clone(),
        updated_at: now.clone(),
        custom_fields: custom_fields.clone(),
        referenced_projects: referenced_projects.clone(),
    };
    let content = generate_frontmatter(&frontmatter, title, description);
    let file_path = issues_dir.join(format!("{issue_id}.md"));
    fs::write(&file_path, &content).await?;
    Ok(OrgIssue {
        id: issue_id,
        display_number,
        title: title.to_string(),
        description: description.to_string(),
        status: status.to_string(),
        priority,
        created_at: now.clone(),
        updated_at: now,
        custom_fields,
        referenced_projects,
    })
}
pub async fn get_org_issue(org_slug: &str, issue_id: &str) -> Result<OrgIssue, OrgIssueError> {
    read_org_issue_file(&get_org_issues_dir(org_slug)?, issue_id).await
}
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
        let issue = match read_org_issue_file(&issues_dir, issue_id).await {
            Ok(i) => i,
            Err(_) => continue,
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
