use std::path::Path;

use crate::item::entities::issue::{get_issue, get_issue_by_display_number, IssueCrudError};
use crate::item::entities::pr::{get_pr, get_pr_by_display_number, PrCrudError};

/// Resolve an issue by display number or UUID.
pub async fn resolve_issue(
    project_path: &Path,
    issue_id: &str,
) -> Result<crate::item::entities::issue::Issue, IssueCrudError> {
    if let Ok(display_num) = issue_id.parse::<u32>() {
        get_issue_by_display_number(project_path, display_num).await
    } else {
        get_issue(project_path, issue_id).await
    }
}

/// Resolve an issue ID (display number or UUID) to a UUID string.
pub async fn resolve_issue_id(
    project_path: &Path,
    issue_id: &str,
) -> Result<String, IssueCrudError> {
    if let Ok(display_num) = issue_id.parse::<u32>() {
        get_issue_by_display_number(project_path, display_num)
            .await
            .map(|issue| issue.id)
    } else {
        Ok(issue_id.to_string())
    }
}

/// Resolve a PR by display number or UUID.
pub async fn resolve_pr(
    project_path: &Path,
    pr_id: &str,
) -> Result<crate::item::entities::pr::PullRequest, PrCrudError> {
    if let Ok(display_num) = pr_id.parse::<u32>() {
        get_pr_by_display_number(project_path, display_num).await
    } else {
        get_pr(project_path, pr_id).await
    }
}

/// Resolve a PR ID (display number or UUID) to a UUID string.
pub async fn resolve_pr_id(project_path: &Path, pr_id: &str) -> Result<String, PrCrudError> {
    if let Ok(display_num) = pr_id.parse::<u32>() {
        get_pr_by_display_number(project_path, display_num)
            .await
            .map(|pr| pr.id)
    } else {
        Ok(pr_id.to_string())
    }
}
