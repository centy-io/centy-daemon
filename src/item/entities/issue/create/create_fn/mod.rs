use super::super::id::generate_issue_id;
use super::super::reconcile::get_next_display_number;
use super::super::status::resolve_issue_status;
use super::helpers::resolve_priority;
use super::types::{CreateIssueOptions, CreateIssueResult, IssueError};
use crate::config::read_config;
use crate::manifest::read_manifest;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;

mod inner;

pub async fn create_issue(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    if options.title.trim().is_empty() {
        return Err(IssueError::TitleRequired);
    }
    let manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueError::NotInitialized)?;
    let issues_path = get_centy_path(project_path).join("issues");
    fs::create_dir_all(&issues_path).await?;
    let issue_id = generate_issue_id();
    let display_number = get_next_display_number(&issues_path).await?;
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);
    let priority = resolve_priority(options.priority, config.as_ref(), priority_levels)?;
    let status = resolve_issue_status(project_path, options.status.clone()).await?;
    let draft = options.draft.unwrap_or(false);
    let now = now_iso();
    let (frontmatter, display_title, body) = inner::build_content(
        project_path,
        display_number,
        &status,
        priority,
        priority_levels,
        &now,
        draft,
        &options,
        options.custom_fields.clone(),
    )
    .await?;
    inner::finalize_issue(
        &issues_path,
        issue_id,
        &frontmatter,
        &display_title,
        &body,
        project_path,
        manifest,
        display_number,
    )
    .await
}
