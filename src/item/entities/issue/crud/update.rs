use super::super::assets::copy_assets_folder;
use super::super::metadata::IssueFrontmatter;
use super::super::priority::validate_priority;
use super::super::status::validate_status_for_project;
use super::read::{read_issue_from_frontmatter, read_issue_from_legacy_folder};
use super::types::{IssueCrudError, UpdateIssueOptions, UpdateIssueResult};
use super::update_helpers::{
    build_issue_struct, build_update_body, build_updated_metadata, AppliedIssueUpdates,
};
use crate::config::read_config;
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::get_centy_path;
use mdstore::generate_frontmatter;
use std::path::Path;
use tokio::fs;

pub async fn update_issue(
    project_path: &Path,
    issue_number: &str,
    options: UpdateIssueOptions,
) -> Result<UpdateIssueResult, IssueCrudError> {
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;
    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    let issue_file_path = issues_path.join(format!("{issue_number}.md"));
    let issue_folder_path = issues_path.join(issue_number);
    let is_new_format = issue_file_path.exists();
    let is_old_format = issue_folder_path.exists();
    if !is_new_format && !is_old_format {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);
    let current = if is_new_format {
        read_issue_from_frontmatter(&issue_file_path, issue_number).await?
    } else {
        read_issue_from_legacy_folder(&issue_folder_path, issue_number).await?
    };
    let old_status = current.metadata.status.clone();
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
    let updates = AppliedIssueUpdates {
        title: new_title,
        description: new_description,
        status: new_status,
        priority: new_priority,
        custom_fields: new_custom_fields,
        draft: options.draft.unwrap_or(current.metadata.draft),
    };
    let updated_metadata = build_updated_metadata(&current, &updates);
    let frontmatter =
        IssueFrontmatter::from_metadata(&updated_metadata, updates.custom_fields.clone());
    let current_content = if is_new_format {
        fs::read_to_string(&issue_file_path).await?
    } else {
        fs::read_to_string(issue_folder_path.join("issue.md")).await?
    };
    let body = build_update_body(
        &old_status,
        &updates.status,
        &updates.description,
        &current_content,
    );
    let issue_content = generate_frontmatter(&frontmatter, &updates.title, &body);
    fs::write(&issue_file_path, &issue_content).await?;
    if is_old_format && !is_new_format {
        let old_assets_path = issue_folder_path.join("assets");
        let new_assets_path = issues_path.join("assets").join(issue_number);
        if old_assets_path.exists() {
            fs::create_dir_all(&new_assets_path).await?;
            copy_assets_folder(&old_assets_path, &new_assets_path)
                .await
                .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;
        }
        fs::remove_dir_all(&issue_folder_path).await?;
    }
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    let issue = build_issue_struct(
        issue_number,
        &updates,
        &current,
        &updated_metadata.common.updated_at,
    );
    let sync_results = if issue.metadata.is_org_issue {
        crate::common::sync_update_to_org_projects(&issue, project_path, None).await
    } else {
        Vec::new()
    };
    Ok(UpdateIssueResult {
        issue,
        manifest,
        sync_results,
    })
}
