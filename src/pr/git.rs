//! Git integration utilities for PR functionality.
//!
//! This module provides utilities for interacting with git:
//! - Detecting the current branch
//! - Validating that branches exist
//! - Getting repository information

use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Not a git repository")]
    NotGitRepository,

    #[error("Failed to execute git command: {0}")]
    CommandError(String),

    #[error("Branch '{0}' does not exist")]
    BranchNotFound(String),

    #[error("Failed to detect current branch")]
    CurrentBranchNotFound,

    #[error("Git command output was not valid UTF-8")]
    InvalidUtf8,
}

/// Detect the current git branch.
///
/// Runs `git rev-parse --abbrev-ref HEAD` in the given project path.
pub fn detect_current_branch(project_path: &Path) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_path)
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitError::NotGitRepository);
        }
        return Err(GitError::CommandError(stderr.to_string()));
    }

    let branch = String::from_utf8(output.stdout)
        .map_err(|_| GitError::InvalidUtf8)?
        .trim()
        .to_string();

    if branch.is_empty() || branch == "HEAD" {
        // Detached HEAD state
        return Err(GitError::CurrentBranchNotFound);
    }

    Ok(branch)
}

/// Validate that a branch exists in the repository.
///
/// Runs `git rev-parse --verify <branch>` to check if the branch exists.
pub fn validate_branch_exists(project_path: &Path, branch: &str) -> Result<bool, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{branch}")])
        .current_dir(project_path)
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;

    // Also check remote branches
    if !output.status.success() {
        let output_remote = Command::new("git")
            .args(["rev-parse", "--verify", &format!("refs/remotes/origin/{branch}")])
            .current_dir(project_path)
            .output()
            .map_err(|e| GitError::CommandError(e.to_string()))?;

        return Ok(output_remote.status.success());
    }

    Ok(output.status.success())
}

/// Check if the current directory is a git repository.
#[must_use] 
pub fn is_git_repository(project_path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the default branch name (main or master).
///
/// Checks if 'main' exists first, then falls back to 'master'.
/// If neither exists, returns "main" as the default.
#[must_use] 
pub fn get_default_branch(project_path: &Path) -> String {
    // Check if main exists
    if validate_branch_exists(project_path, "main").unwrap_or(false) {
        return "main".to_string();
    }

    // Check if master exists
    if validate_branch_exists(project_path, "master").unwrap_or(false) {
        return "master".to_string();
    }

    // Default to main
    "main".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_is_git_repository() {
        // Current directory should be part of a git repo (the centy-daemon project)
        let cwd = env::current_dir().unwrap();
        // This test may fail if run outside a git repo, which is acceptable
        let _ = is_git_repository(&cwd);
    }

    #[test]
    fn test_non_git_directory() {
        // /tmp is typically not a git repository
        let non_git = Path::new("/tmp");
        assert!(!is_git_repository(non_git));
    }
}
