//! Git repository utilities for user extraction.
//!
//! Provides functionality to extract contributor information from git history.

use super::types::{GitContributor, UserError};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// Check if a path is inside a git repository
pub fn is_git_repository(project_path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_path)
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get unique contributors from git history
pub fn get_git_contributors(project_path: &Path) -> Result<Vec<GitContributor>, UserError> {
    let output = Command::new("git")
        .args(["log", "--format=%an|%ae"])
        .current_dir(project_path)
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| UserError::GitError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UserError::GitError(stderr.to_string()));
    }

    let log_output = String::from_utf8(output.stdout)
        .map_err(|_| UserError::GitError("Invalid UTF-8 in git log output".to_string()))?;

    // Use a HashSet to deduplicate by email (case-insensitive)
    let mut seen_emails: HashSet<String> = HashSet::new();
    let mut contributors: Vec<GitContributor> = Vec::new();

    for line in log_output.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if let (Some(name_part), Some(email_part)) = (parts.first(), parts.get(1)) {
            let name = name_part.trim().to_string();
            let email = email_part.trim().to_string();
            let email_lower = email.to_lowercase();

            if !email.is_empty() && !seen_emails.contains(&email_lower) {
                seen_emails.insert(email_lower);
                contributors.push(GitContributor { name, email });
            }
        }
    }

    Ok(contributors)
}
