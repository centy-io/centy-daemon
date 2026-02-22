//! CRUD operations for organization-level issues.
//!
//! Org issues are stored at ~/.centy/orgs/{slug}/issues/{uuid}.md

use super::paths::{get_org_issues_dir, PathError};
use crate::item::entities::issue::org_registry::{
    get_next_org_display_number, OrgIssueRegistryError,
};
use crate::utils::now_iso;
use mdstore::{generate_frontmatter, parse_frontmatter, FrontmatterError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum OrgIssueError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML frontmatter error: {0}")]
    FrontmatterError(#[from] FrontmatterError),

    #[error("Path error: {0}")]
    PathError(#[from] PathError),

    #[error("Org registry error: {0}")]
    OrgRegistryError(#[from] OrgIssueRegistryError),

    #[error("Org issue not found: {0}")]
    NotFound(String),

    #[error("Title is required")]
    TitleRequired,
}

/// Frontmatter for org issue markdown files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrgIssueFrontmatter {
    pub display_number: u32,
    pub status: String,
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub referenced_projects: Vec<String>,
}

/// A fully loaded org issue
#[derive(Debug, Clone)]
pub struct OrgIssue {
    pub id: String,
    pub display_number: u32,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    pub custom_fields: HashMap<String, String>,
    pub referenced_projects: Vec<String>,
}

impl OrgIssue {}

/// Options for listing org issues
#[derive(Debug, Clone, Default)]
pub struct ListOrgIssuesOptions {
    /// Filter by status (None = all)
    pub status: Option<String>,
    /// Filter by priority (None = all)
    pub priority: Option<u32>,
    /// Filter by referenced project path (None = all)
    pub referenced_project: Option<String>,
}

/// Options for updating an org issue
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

/// Read a single org issue file
async fn read_org_issue_file(issues_dir: &Path, issue_id: &str) -> Result<OrgIssue, OrgIssueError> {
    let file_path = issues_dir.join(format!("{issue_id}.md"));

    if !file_path.exists() {
        return Err(OrgIssueError::NotFound(issue_id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    parse_org_issue_content(&content, issue_id)
}

/// Parse org issue content from markdown with frontmatter
fn parse_org_issue_content(content: &str, issue_id: &str) -> Result<OrgIssue, OrgIssueError> {
    let (frontmatter, title, description) = parse_frontmatter::<OrgIssueFrontmatter>(content)?;

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

/// Create a new org issue
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

/// Get an org issue by UUID
pub async fn get_org_issue(org_slug: &str, issue_id: &str) -> Result<OrgIssue, OrgIssueError> {
    let issues_dir = get_org_issues_dir(org_slug)?;
    read_org_issue_file(&issues_dir, issue_id).await
}

/// Get an org issue by display number
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

/// List org issues with optional filtering
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

        // Apply filters
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

    // Sort by display_number ascending
    issues.sort_by_key(|i| i.display_number);

    Ok(issues)
}

/// Update an org issue
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

    // Add referenced projects
    for project in opts.add_referenced_projects {
        if !issue.referenced_projects.contains(&project) {
            issue.referenced_projects.push(project);
        }
    }

    // Remove referenced projects
    issue
        .referenced_projects
        .retain(|p| !opts.remove_referenced_projects.contains(p));

    issue.updated_at = now_iso();

    // Write back to file
    let frontmatter = OrgIssueFrontmatter {
        display_number: issue.display_number,
        status: issue.status.clone(),
        priority: issue.priority,
        created_at: issue.created_at.clone(),
        updated_at: issue.updated_at.clone(),
        custom_fields: issue.custom_fields.clone(),
        referenced_projects: issue.referenced_projects.clone(),
    };

    let content = generate_frontmatter(&frontmatter, &issue.title, &issue.description);
    let file_path = issues_dir.join(format!("{issue_id}.md"));
    fs::write(&file_path, &content).await?;

    Ok(issue)
}

/// Delete an org issue
pub async fn delete_org_issue(org_slug: &str, issue_id: &str) -> Result<(), OrgIssueError> {
    let issues_dir = get_org_issues_dir(org_slug)?;
    let file_path = issues_dir.join(format!("{issue_id}.md"));

    if !file_path.exists() {
        return Err(OrgIssueError::NotFound(issue_id.to_string()));
    }

    fs::remove_file(&file_path).await?;
    Ok(())
}
