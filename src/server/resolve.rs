use std::path::Path;

use crate::item::entities::issue::{get_issue, get_issue_by_display_number, IssueCrudError};

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
