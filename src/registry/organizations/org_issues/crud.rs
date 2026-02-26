//! Update and delete operations for org issues.
use super::crud_ops::read_org_issue_file;
use super::crud_types::{OrgIssue, OrgIssueError, OrgIssueFrontmatter};
use super::paths::get_org_issues_dir;
use crate::utils::now_iso;
use mdstore::generate_frontmatter;
use std::collections::HashMap;
use tokio::fs;

#[derive(Debug, Clone, Default)]
pub struct UpdateOrgIssueOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<u32>,
    pub custom_fields: Option<HashMap<String, String>>,
    pub add_referenced_projects: Vec<String>,
    pub remove_referenced_projects: Vec<String>,
}

pub async fn update_org_issue(
    org_slug: &str,
    issue_id: &str,
    opts: UpdateOrgIssueOptions,
) -> Result<OrgIssue, OrgIssueError> {
    let issues_dir = get_org_issues_dir(org_slug)?;
    let mut issue = read_org_issue_file(&issues_dir, issue_id).await?;
    if let Some(title) = opts.title {
        issue.title = title;
    }
    if let Some(description) = opts.description {
        issue.description = description;
    }
    if let Some(status) = opts.status {
        issue.status = status;
    }
    if let Some(priority) = opts.priority {
        issue.priority = priority;
    }
    if let Some(custom_fields) = opts.custom_fields {
        issue.custom_fields = custom_fields;
    }
    for project in opts.add_referenced_projects {
        if !issue.referenced_projects.contains(&project) {
            issue.referenced_projects.push(project);
        }
    }
    issue
        .referenced_projects
        .retain(|p| !opts.remove_referenced_projects.contains(p));
    issue.updated_at = now_iso();
    let frontmatter = OrgIssueFrontmatter {
        display_number: issue.display_number,
        status: issue.status.clone(),
        priority: issue.priority,
        created_at: issue.created_at.clone(),
        updated_at: issue.updated_at.clone(),
        custom_fields: issue.custom_fields.clone(),
        referenced_projects: issue.referenced_projects.clone(),
    };
    fs::write(
        issues_dir.join(format!("{issue_id}.md")),
        generate_frontmatter(&frontmatter, &issue.title, &issue.description),
    )
    .await?;
    Ok(issue)
}

pub async fn delete_org_issue(org_slug: &str, issue_id: &str) -> Result<(), OrgIssueError> {
    let file_path = get_org_issues_dir(org_slug)?.join(format!("{issue_id}.md"));
    if !file_path.exists() {
        return Err(OrgIssueError::NotFound(issue_id.to_string()));
    }
    fs::remove_file(&file_path).await?;
    Ok(())
}
