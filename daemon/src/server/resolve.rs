use std::path::Path;

use crate::item::entities::issue::{get_issue, get_issue_by_display_number, IssueCrudError};

/// Resolve an issue by display number, UUID, or `centy:`-prefixed reference.
///
/// Accepted formats:
/// - `"42"` — display number
/// - `"6f4853a9-3d82-4013-b909-c2d637f44541"` — internal UUID
/// - `"centy:42"` — `centy:` prefix with display number
/// - `"centy:6f4853a9-3d82-4013-b909-c2d637f44541"` — `centy:` prefix with UUID
pub async fn resolve_issue(
    project_path: &Path,
    issue_id: &str,
) -> Result<crate::item::entities::issue::Issue, IssueCrudError> {
    let id = issue_id.strip_prefix("centy:").unwrap_or(issue_id);
    if let Ok(display_num) = id.parse::<u32>() {
        get_issue_by_display_number(project_path, display_num).await
    } else {
        get_issue(project_path, id).await
    }
}
