use super::super::assets::copy_assets_folder;
use super::super::metadata::IssueMetadata;
use super::super::planning::{add_planning_note, has_planning_note, is_planning_status};
use super::super::priority::validate_priority;
use super::super::status::validate_status_for_project;
use super::types::{Issue, IssueCrudError, IssueMetadataFlat, UpdateIssueOptions};
use crate::utils::now_iso;
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

pub fn build_updated_metadata(current: &Issue, updates: &AppliedIssueUpdates) -> IssueMetadata {
    IssueMetadata {
        common: mdstore::CommonMetadata {
            display_number: current.metadata.display_number,
            status: updates.status.clone(),
            priority: updates.priority,
            created_at: current.metadata.created_at.clone(),
            updated_at: now_iso(),
            custom_fields: updates
                .custom_fields
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        },
        draft: updates.draft,
        deleted_at: current.metadata.deleted_at.clone(),
        is_org_issue: current.metadata.is_org_issue,
        org_slug: current.metadata.org_slug.clone(),
        org_display_number: current.metadata.org_display_number,
    }
}

pub fn build_update_body(
    old_status: &str,
    new_status: &str,
    description: &str,
    current_content: &str,
) -> String {
    if is_planning_status(old_status) && is_planning_status(new_status) {
        if has_planning_note(current_content) {
            add_planning_note(description)
        } else {
            description.to_string()
        }
    } else if !is_planning_status(old_status) && is_planning_status(new_status) {
        add_planning_note(description)
    } else {
        description.to_string()
    }
}

pub async fn compute_sync_results(
    issue: &Issue,
    project_path: &Path,
) -> Vec<crate::common::OrgSyncResult> {
    if issue.metadata.is_org_issue {
        crate::common::sync_update_to_org_projects(issue, project_path, None).await
    } else {
        Vec::new()
    }
}

pub fn build_issue_struct(
    issue_number: &str,
    updates: &AppliedIssueUpdates,
    current: &Issue,
    updated_at: &str,
) -> Issue {
    #[allow(deprecated)]
    Issue {
        id: issue_number.to_string(),
        issue_number: issue_number.to_string(),
        title: updates.title.clone(),
        description: updates.description.clone(),
        metadata: IssueMetadataFlat {
            display_number: current.metadata.display_number,
            status: updates.status.clone(),
            priority: updates.priority,
            created_at: current.metadata.created_at.clone(),
            updated_at: updated_at.to_string(),
            custom_fields: updates.custom_fields.clone(),
            draft: updates.draft,
            deleted_at: current.metadata.deleted_at.clone(),
            is_org_issue: current.metadata.is_org_issue,
            org_slug: current.metadata.org_slug.clone(),
            org_display_number: current.metadata.org_display_number,
        },
    }
}
