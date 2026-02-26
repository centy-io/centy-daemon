use super::types::{CreateIssueOptions, CreateIssueResult, IssueError};
use std::path::Path;
use tokio::fs;

/// Get the next issue number (zero-padded to 4 digits).
/// DEPRECATED: Use UUID-based folders with display_number in metadata.
#[deprecated(note = "Use UUID-based folders with display_number in metadata")]
#[allow(unknown_lints, max_nesting_depth)]
pub async fn get_next_issue_number(issues_path: &Path) -> Result<String, std::io::Error> {
    if !issues_path.exists() {
        return Ok("0001".to_string());
    }

    let mut max_number: u32 = 0;

    let mut entries = fs::read_dir(issues_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(num) = name.parse::<u32>() {
                    max_number = max_number.max(num);
                }
            }
        }
    }

    Ok(format!("{:04}", max_number.saturating_add(1)))
}

/// Thin wrapper around `create_issue` for backward compatibility.
pub async fn create_issue_with_title_generation(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    super::create_issue(project_path, options).await
}
