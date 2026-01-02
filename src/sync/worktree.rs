//! Sync worktree management.
//!
//! This module manages the persistent worktree at `~/.centy/sync/{hash}/`
//! that is always checked out to the centy branch.

use super::{SyncError, CENTY_BRANCH};
use crate::utils::compute_hash;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// Get the sync worktree path for a project.
///
/// Returns: `~/.centy/sync/{sha256(canonical_project_path)[0:16]}/`
pub fn get_sync_worktree_path(project_path: &Path) -> Result<PathBuf, SyncError> {
    let home_dir = dirs::home_dir().ok_or(SyncError::HomeDirNotFound)?;

    // Get canonical path for consistent hashing
    let canonical = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());

    let path_str = canonical.to_string_lossy();
    let hash = compute_hash(&path_str);
    let short_hash = &hash[..16];

    Ok(home_dir.join(".centy").join("sync").join(short_hash))
}

/// Check if sync worktree exists and is valid
pub fn sync_worktree_exists(project_path: &Path) -> Result<bool, SyncError> {
    let worktree_path = get_sync_worktree_path(project_path)?;

    if !worktree_path.exists() {
        return Ok(false);
    }

    // Check if it's a valid git worktree
    let git_file = worktree_path.join(".git");
    if !git_file.exists() {
        return Ok(false);
    }

    // Verify it's on the centy branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&worktree_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Ok(false);
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch == CENTY_BRANCH)
}

/// Ensure sync worktree exists, creating if necessary.
///
/// This function:
/// 1. Creates `~/.centy/sync/` directory if not exists
/// 2. Creates the worktree if missing
/// 3. Verifies the worktree is on the centy branch
pub async fn ensure_sync_worktree(project_path: &Path) -> Result<PathBuf, SyncError> {
    let worktree_path = get_sync_worktree_path(project_path)?;

    // Check if worktree already exists and is valid
    if sync_worktree_exists(project_path)? {
        return Ok(worktree_path);
    }

    // Create parent directories
    if let Some(parent) = worktree_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // If path exists but isn't a valid worktree, remove it
    if worktree_path.exists() {
        fs::remove_dir_all(&worktree_path).await?;
    }

    // Create the worktree
    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            &worktree_path.to_string_lossy(),
            CENTY_BRANCH,
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SyncError::WorktreeError(format!(
            "Failed to create sync worktree: {stderr}"
        )));
    }

    Ok(worktree_path)
}

/// Remove sync worktree and prune references
pub async fn remove_sync_worktree(project_path: &Path) -> Result<(), SyncError> {
    let worktree_path = get_sync_worktree_path(project_path)?;

    if !worktree_path.exists() {
        return Ok(());
    }

    // Remove the worktree
    let output = Command::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            &worktree_path.to_string_lossy(),
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If worktree doesn't exist in git's tracking, just remove the directory
        if stderr.contains("is not a working tree") {
            fs::remove_dir_all(&worktree_path).await?;
            return Ok(());
        }
        return Err(SyncError::WorktreeError(format!(
            "Failed to remove sync worktree: {stderr}"
        )));
    }

    // Prune any stale worktree references
    let _ = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(project_path)
        .output();

    Ok(())
}

/// Get the .centy path within sync worktree.
///
/// Returns: `{sync_worktree}/.centy/`
pub fn get_sync_centy_path(project_path: &Path) -> Result<PathBuf, SyncError> {
    let worktree_path = get_sync_worktree_path(project_path)?;
    Ok(worktree_path.join(".centy"))
}

/// Validate that a worktree is properly set up
pub fn validate_worktree(worktree_path: &Path) -> Result<bool, SyncError> {
    // Check .git file exists (worktrees have a .git file, not directory)
    let git_file = worktree_path.join(".git");
    if !git_file.exists() {
        return Ok(false);
    }

    // Check we can run git commands
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    Ok(output.status.success())
}

/// Repair a corrupted worktree by recreating it
pub async fn repair_worktree(project_path: &Path) -> Result<PathBuf, SyncError> {
    // Remove existing worktree
    remove_sync_worktree(project_path).await?;

    // Prune any stale references
    let _ = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(project_path)
        .output();

    // Recreate
    ensure_sync_worktree(project_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sync_worktree_path() {
        let project_path = Path::new("/home/user/my-project");
        let result = get_sync_worktree_path(project_path);

        // Should succeed and return a path under ~/.centy/sync/
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".centy/sync/"));
    }

    #[test]
    fn test_get_sync_worktree_path_consistent_hash() {
        let project_path = Path::new("/home/user/my-project");
        let path1 = get_sync_worktree_path(project_path).unwrap();
        let path2 = get_sync_worktree_path(project_path).unwrap();

        // Same project should always get same hash
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_get_sync_worktree_path_different_projects() {
        let project1 = Path::new("/home/user/project1");
        let project2 = Path::new("/home/user/project2");

        let path1 = get_sync_worktree_path(project1).unwrap();
        let path2 = get_sync_worktree_path(project2).unwrap();

        // Different projects should get different hashes
        assert_ne!(path1, path2);
    }

    #[test]
    fn test_get_sync_centy_path() {
        let project_path = Path::new("/home/user/my-project");
        let result = get_sync_centy_path(project_path);

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with(".centy"));
    }
}
