use super::super::metadata::IssueFrontmatter;
use super::read::{read_issue_from_frontmatter, read_issue_from_legacy_folder};
use super::types::{IssueCrudError, UpdateIssueOptions, UpdateIssueResult};
use super::update_helpers::{
    build_issue_struct, build_update_body, build_updated_metadata, compute_sync_results,
    migrate_legacy_format, resolve_update_options,
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
    let issues_path = get_centy_path(project_path).join("issues");
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
    let updates = resolve_update_options(&current, options, project_path, priority_levels).await?;
    let updated_metadata = build_updated_metadata(&current, &updates);
    let frontmatter =
        IssueFrontmatter::from_metadata(&updated_metadata, updates.custom_fields.clone());
    let current_content = if is_new_format {
        fs::read_to_string(&issue_file_path).await?
    } else {
        fs::read_to_string(issue_folder_path.join("issue.md")).await?
    };
    let body = build_update_body(
        &current.metadata.status,
        &updates.status,
        &updates.description,
        &current_content,
    );
    let issue_content = generate_frontmatter(&frontmatter, &updates.title, &body);
    fs::write(&issue_file_path, &issue_content).await?;
    if is_old_format && !is_new_format {
        migrate_legacy_format(&issue_folder_path, &issues_path, issue_number).await?;
    }
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    let issue = build_issue_struct(
        issue_number,
        &updates,
        &current,
        &updated_metadata.common.updated_at,
    );
    let sync_results = compute_sync_results(&issue, project_path).await;
    Ok(UpdateIssueResult {
        issue,
        manifest,
        sync_results,
    })
}
