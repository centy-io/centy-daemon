use std::path::Path;

use crate::item::entities::issue::{get_issue, get_issue_by_display_number};
use crate::item::entities::pr::{get_pr, get_pr_by_display_number};

/// Resolve an issue by display number or UUID.
pub async fn resolve_issue(
    project_path: &Path,
    issue_id: &str,
) -> Result<crate::item::entities::issue::Issue, String> {
    if let Ok(display_num) = issue_id.parse::<u32>() {
        get_issue_by_display_number(project_path, display_num)
            .await
            .map_err(|e| format!("Issue not found: {e}"))
    } else {
        get_issue(project_path, issue_id)
            .await
            .map_err(|e| format!("Issue not found: {e}"))
    }
}

/// Resolve an issue ID (display number or UUID) to a UUID string.
pub async fn resolve_issue_id(project_path: &Path, issue_id: &str) -> Result<String, String> {
    if let Ok(display_num) = issue_id.parse::<u32>() {
        get_issue_by_display_number(project_path, display_num)
            .await
            .map(|issue| issue.id)
            .map_err(|e| format!("Issue not found: {e}"))
    } else {
        Ok(issue_id.to_string())
    }
}

/// Resolve a PR by display number or UUID.
pub async fn resolve_pr(
    project_path: &Path,
    pr_id: &str,
) -> Result<crate::item::entities::pr::PullRequest, String> {
    if let Ok(display_num) = pr_id.parse::<u32>() {
        get_pr_by_display_number(project_path, display_num)
            .await
            .map_err(|e| format!("PR not found: {e}"))
    } else {
        get_pr(project_path, pr_id)
            .await
            .map_err(|e| format!("PR not found: {e}"))
    }
}

/// Resolve a PR ID (display number or UUID) to a UUID string.
pub async fn resolve_pr_id(project_path: &Path, pr_id: &str) -> Result<String, String> {
    if let Ok(display_num) = pr_id.parse::<u32>() {
        get_pr_by_display_number(project_path, display_num)
            .await
            .map(|pr| pr.id)
            .map_err(|e| format!("PR not found: {e}"))
    } else {
        Ok(pr_id.to_string())
    }
}
