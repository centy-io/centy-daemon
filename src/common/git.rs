//! Git integration utilities.
//!
//! This module provides utilities for interacting with git:
//! - Detecting the current branch
//! - Validating that branches exist
//! - Getting repository information
//!
//! ## Worktree Functions
//!
//! The worktree functions (`create_worktree`, `remove_worktree`, `prune_worktrees`)
//! are kept for backwards compatibility. For new code, prefer using
//! `crate::workspace::gwq_client::GwqClient` which provides a more complete
//! worktree management API via the gwq CLI tool.

#![allow(dead_code)]

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

    #[error("Worktree error: {0}")]
    WorktreeError(String),

    #[error("Remote '{0}' not found")]
    RemoteNotFound(String),
}

/// Detect the current git branch.
///
/// Runs `git rev-parse --abbrev-ref HEAD` in the given project path.
pub fn detect_current_branch(project_path: &Path) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_path)
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
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
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;

    // Also check remote branches
    if !output.status.success() {
        let output_remote = Command::new("git")
            .args([
                "rev-parse",
                "--verify",
                &format!("refs/remotes/origin/{branch}"),
            ])
            .current_dir(project_path)
            // Clear GIT_DIR to avoid being affected by git hooks environment
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
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
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the origin remote URL from a git repository.
///
/// Runs `git remote get-url origin` in the given project path.
pub fn get_remote_origin_url(project_path: &Path) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(project_path)
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitError::NotGitRepository);
        }
        return Err(GitError::RemoteNotFound("origin".to_string()));
    }

    String::from_utf8(output.stdout)
        .map_err(|_| GitError::InvalidUtf8)
        .map(|s| s.trim().to_string())
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

/// Create a detached git worktree at the target path.
///
/// This creates a lightweight checkout of HEAD without uncommitted changes,
/// ideal for isolated work on issues.
///
/// # Arguments
/// * `source_path` - Path to the source git repository
/// * `target_path` - Path where the worktree will be created
/// * `git_ref` - Git ref to check out (usually "HEAD")
///
/// # Deprecated
/// For new code, prefer using `crate::workspace::gwq_client::GwqClient::add_worktree_at_path`.
pub fn create_worktree(
    source_path: &Path,
    target_path: &Path,
    git_ref: &str,
) -> Result<(), GitError> {
    // First verify source is a git repo
    if !is_git_repository(source_path) {
        return Err(GitError::NotGitRepository);
    }

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "--detach",
            &target_path.to_string_lossy(),
            git_ref,
        ])
        .current_dir(source_path)
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::WorktreeError(stderr.to_string()));
    }

    Ok(())
}

/// Remove a git worktree.
///
/// Uses `--force` to ensure removal even if there are uncommitted changes
/// in the worktree (since it's meant to be temporary anyway).
///
/// # Arguments
/// * `source_path` - Path to the source git repository (not the worktree)
/// * `worktree_path` - Path to the worktree to remove
///
/// # Deprecated
/// For new code, prefer using `crate::workspace::gwq_client::GwqClient::remove_worktree_from_repo`.
pub fn remove_worktree(source_path: &Path, worktree_path: &Path) -> Result<(), GitError> {
    let output = Command::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            &worktree_path.to_string_lossy(),
        ])
        .current_dir(source_path)
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If worktree doesn't exist, that's okay for our cleanup purposes
        if !stderr.contains("is not a working tree") {
            return Err(GitError::WorktreeError(stderr.to_string()));
        }
    }

    Ok(())
}

/// Prune stale worktree references.
///
/// Call this after manually deleting worktree directories to clean up
/// git's internal tracking.
///
/// # Deprecated
/// For new code, prefer using `crate::workspace::gwq_client::GwqClient::prune`.
pub fn prune_worktrees(source_path: &Path) -> Result<(), GitError> {
    let output = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(source_path)
        // Clear GIT_DIR to avoid being affected by git hooks environment
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::WorktreeError(stderr.to_string()));
    }

    Ok(())
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
        // Use root directory which is definitely not inside a git repository
        // (git won't traverse above /)
        let non_git = Path::new("/");
        assert!(!is_git_repository(non_git));
    }
}
