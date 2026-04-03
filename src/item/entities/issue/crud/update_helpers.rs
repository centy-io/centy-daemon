use super::super::assets::copy_assets_folder;
use super::super::priority::validate_priority;
use super::super::status::validate_status_for_project;
use super::types::{Issue, IssueCrudError, UpdateIssueOptions};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

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

pub async fn migrate_legacy_format(
    issue_folder_path: &Path,
    issues_path: &Path,
    issue_number: &str,
) -> Result<(), IssueCrudError> {
    let old_assets_path = issue_folder_path.join("assets");
    if old_assets_path.exists() {
        let new_assets_path = issues_path.join("assets").join(issue_number);
        fs::create_dir_all(&new_assets_path).await?;
        copy_assets_folder(&old_assets_path, &new_assets_path)
            .await
            .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;
    }
    fs::remove_dir_all(issue_folder_path).await?;
    Ok(())
}
