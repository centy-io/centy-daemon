use super::read::read_issue_from_frontmatter;
use super::types::{IssueCrudError, UpdateIssueOptions, UpdateIssueResult};
use super::update_builders::{build_issue_struct, build_update_body, build_updated_frontmatter};
use super::update_helpers::resolve_update_options;
use crate::config::read_config;
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{get_centy_path, CENTY_HEADER_YAML};
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
    if !issue_file_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);
    let current = read_issue_from_frontmatter(&issue_file_path, issue_number).await?;
    let updates = resolve_update_options(&current, options, project_path, priority_levels).await?;
    let frontmatter = build_updated_frontmatter(&current, &updates);
    let current_content = fs::read_to_string(&issue_file_path).await?;
    let body = build_update_body(
        &current.metadata.status,
        &updates.status,
        &updates.description,
        &current_content,
    );
    let issue_content =
        generate_frontmatter(&frontmatter, &updates.title, &body, Some(CENTY_HEADER_YAML));
    fs::write(&issue_file_path, &issue_content).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    let issue = build_issue_struct(issue_number, &updates, &current);
    Ok(UpdateIssueResult { issue, manifest })
}
