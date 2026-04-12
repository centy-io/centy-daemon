//! I/O helpers for `move_issue`: loading source and removing files.
use super::read::read_issue_from_frontmatter;
use super::types::{Issue, IssueCrudError};
use crate::config::read_config;
use crate::utils::get_centy_path;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Read the issue from disk.
/// Returns `(issue, file_path, assets_path)`.
pub(super) async fn load_source_issue(
    source_issues_path: &Path,
    issue_id: &str,
) -> Result<(Issue, PathBuf, PathBuf), IssueCrudError> {
    let file_path = source_issues_path.join(format!("{issue_id}.md"));
    let assets_path = source_issues_path.join("assets").join(issue_id);
    if file_path.exists() {
        let issue = read_issue_from_frontmatter(&file_path, issue_id).await?;
        Ok((issue, file_path, assets_path))
    } else {
        Err(IssueCrudError::IssueNotFound(issue_id.to_string()))
    }
}

/// Validate that the source issue can be moved to the target project.
pub(super) async fn validate_issue_move(
    source_issue: &Issue,
    target_project_path: &Path,
) -> Result<(), IssueCrudError> {
    let target_config = read_config(target_project_path).await.ok().flatten();
    let target_priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);
    if source_issue.metadata.priority > target_priority_levels {
        return Err(IssueCrudError::InvalidPriorityInTarget(
            source_issue.metadata.priority,
        ));
    }
    super::super::status::validate_status_for_project(
        target_project_path,
        "issues",
        &source_issue.metadata.status,
    )
    .await?;
    Ok(())
}

/// Remove the source issue files from disk.
pub(super) async fn remove_source_issue(
    file_path: &Path,
    assets_path: &Path,
) -> std::io::Result<()> {
    fs::remove_file(file_path).await?;
    if assets_path.exists() {
        fs::remove_dir_all(assets_path).await?;
    }
    Ok(())
}

/// Compute the source project's issues directory path.
pub(super) fn source_issues_path(source_project_path: &Path) -> PathBuf {
    get_centy_path(source_project_path).join("issues")
}
