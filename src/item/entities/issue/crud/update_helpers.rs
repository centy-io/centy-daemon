use super::super::priority::validate_priority;
use super::super::status::validate_status_for_project;
use super::types::{Issue, IssueCrudError, UpdateIssueOptions};
use std::collections::HashMap;
use std::path::Path;

pub struct AppliedIssueUpdates {
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: u32,
    pub custom_fields: HashMap<String, String>,
    pub draft: bool,
}

pub async fn resolve_update_options(
    current: &Issue,
    options: UpdateIssueOptions,
    project_path: &Path,
    priority_levels: u32,
) -> Result<AppliedIssueUpdates, IssueCrudError> {
    let new_title = options.title.unwrap_or_else(|| current.title.clone());
    let new_description = options
        .description
        .unwrap_or_else(|| current.description.clone());
    let new_status = options
        .status
        .unwrap_or_else(|| current.metadata.status.clone());
    validate_status_for_project(project_path, "issues", &new_status).await?;
    let new_priority = if let Some(p) = options.priority {
        validate_priority(p, priority_levels)?;
        p
    } else {
        current.metadata.priority
    };
    let mut new_custom_fields = current.metadata.custom_fields.clone();
    for (key, value) in options.custom_fields {
        new_custom_fields.insert(key, value);
    }
    Ok(AppliedIssueUpdates {
        title: new_title,
        description: new_description,
        status: new_status,
        priority: new_priority,
        custom_fields: new_custom_fields,
        draft: options.draft.unwrap_or(current.metadata.draft),
    })
}

