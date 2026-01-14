//! Workspace path generation and manipulation.
//!
//! Provides functions for generating unique workspace paths and sanitizing
//! project names for use in filesystem paths.

use chrono::{Duration, Utc};
use std::path::{Path, PathBuf};

/// Extract and sanitize project name from a path.
///
/// - Uses the last directory component
/// - Replaces non-alphanumeric chars with hyphens
/// - Converts to lowercase
/// - Truncates to max 30 chars
/// - Removes leading/trailing hyphens
pub fn sanitize_project_name(project_path: &Path) -> String {
    let name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    // Remove consecutive hyphens and trim
    let mut result = String::new();
    let mut prev_hyphen = true; // Start true to skip leading hyphens
    for c in sanitized.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    // Remove trailing hyphen and truncate
    let result = result.trim_end_matches('-');
    if result.len() > 30 {
        // Find a clean break point (avoid cutting mid-word)
        let truncated = &result[..30];
        truncated
            .rfind('-')
            .map(|i| &truncated[..i])
            .unwrap_or(truncated)
            .to_string()
    } else {
        result.to_string()
    }
}

/// Generate a unique workspace path in the system temp directory.
///
/// Format: `{project_name}-issue-{display_number}-{short_timestamp}`
/// Example: `my-app-issue-42-20231224`
pub fn generate_workspace_path(project_path: &Path, issue_display_number: u32) -> PathBuf {
    let project_name = sanitize_project_name(project_path);
    let date = Utc::now().format("%Y%m%d").to_string();

    let workspace_name = format!("{project_name}-issue-{issue_display_number}-{date}");
    std::env::temp_dir().join(workspace_name)
}

/// Calculate expiration timestamp based on TTL in hours.
///
/// Returns an RFC3339 formatted timestamp string.
pub fn calculate_expires_at(ttl_hours: u32) -> String {
    let expires = Utc::now() + Duration::hours(i64::from(ttl_hours));
    expires.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_project_name_simple() {
        let path = Path::new("/home/user/my-project");
        assert_eq!(sanitize_project_name(path), "my-project");
    }

    #[test]
    fn test_sanitize_project_name_with_spaces() {
        let path = Path::new("/home/user/My Cool Project");
        assert_eq!(sanitize_project_name(path), "my-cool-project");
    }

    #[test]
    fn test_sanitize_project_name_special_chars() {
        let path = Path::new("/home/user/project_v2.0@beta!");
        assert_eq!(sanitize_project_name(path), "project-v2-0-beta");
    }

    #[test]
    fn test_sanitize_project_name_long_name() {
        let path = Path::new("/home/user/this-is-a-very-long-project-name-that-exceeds-limit");
        let result = sanitize_project_name(path);
        assert!(result.len() <= 30);
        // Should break at hyphen boundary
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn test_sanitize_project_name_leading_special() {
        let path = Path::new("/home/user/---project---");
        assert_eq!(sanitize_project_name(path), "project");
    }

    #[test]
    fn test_generate_workspace_path() {
        let project_path = Path::new("/home/user/my-app");
        let path = generate_workspace_path(project_path, 42);
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("my-app-issue-42-"));
        // Should contain date in YYYYMMDD format
        assert!(path_str.contains(&Utc::now().format("%Y%m%d").to_string()));
    }

    #[test]
    fn test_generate_workspace_path_complex_name() {
        let project_path = Path::new("/Users/dev/My Cool App");
        let path = generate_workspace_path(project_path, 1);
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("my-cool-app-issue-1-"));
    }

    #[test]
    fn test_calculate_expires_at() {
        let expires = calculate_expires_at(12);
        // Should be a valid RFC3339 timestamp
        assert!(expires.contains('T'));
        assert!(expires.contains('+') || expires.contains('Z'));
    }
}
