//! Orphan branch operations for centy sync.
//!
//! This module handles creating and managing the orphan `centy` branch
//! that stores all `.centy/` data separately from code branches.

use super::{SyncError, CENTY_BRANCH};
use std::path::Path;
use std::process::Command;

/// Result of a pull operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullResult {
    /// Already up to date
    UpToDate,
    /// Fast-forward merge was performed
    FastForward,
    /// A merge was performed
    Merged,
    /// There were conflicts that need resolution
    Conflict { files: Vec<String> },
}

/// Check if the centy branch exists locally
pub fn centy_branch_exists(project_path: &Path) -> Result<bool, SyncError> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{CENTY_BRANCH}")])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    Ok(output.status.success())
}

/// Check if remote origin/centy exists
pub fn remote_centy_branch_exists(project_path: &Path) -> Result<bool, SyncError> {
    // First fetch to ensure we have latest refs
    let _ = Command::new("git")
        .args(["fetch", "origin", "--prune"])
        .current_dir(project_path)
        .output();

    let output = Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            &format!("refs/remotes/origin/{CENTY_BRANCH}"),
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    Ok(output.status.success())
}

/// Create orphan centy branch with initial .centy content.
///
/// This creates a branch with no parent commits, containing only the .centy directory.
/// The branch is created in a temporary worktree to avoid affecting the current checkout.
pub fn create_orphan_centy_branch(project_path: &Path) -> Result<(), SyncError> {
    // Save current branch
    let current_branch = get_current_branch(project_path)?;

    // Check if .centy exists
    let centy_path = project_path.join(".centy");
    if !centy_path.exists() {
        return Err(SyncError::GitCommandFailed(
            ".centy directory does not exist".to_string(),
        ));
    }

    // Create orphan branch
    let output = Command::new("git")
        .args(["checkout", "--orphan", CENTY_BRANCH])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Try to return to original branch before returning error
        let _ = Command::new("git")
            .args(["checkout", &current_branch])
            .current_dir(project_path)
            .output();
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to create orphan branch: {stderr}"
        )));
    }

    // Remove all files from index (but not from working tree)
    let output = Command::new("git")
        .args(["rm", "-rf", "--cached", "."])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Ignore errors if nothing to remove
        if !stderr.contains("did not match any files") {
            let _ = Command::new("git")
                .args(["checkout", &current_branch])
                .current_dir(project_path)
                .output();
            return Err(SyncError::GitCommandFailed(format!(
                "Failed to clear index: {stderr}"
            )));
        }
    }

    // Add only .centy directory
    let output = Command::new("git")
        .args(["add", ".centy"])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = Command::new("git")
            .args(["checkout", &current_branch])
            .current_dir(project_path)
            .output();
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to add .centy: {stderr}"
        )));
    }

    // Commit
    let output = Command::new("git")
        .args(["commit", "-m", "Initial centy branch"])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = Command::new("git")
            .args(["checkout", &current_branch])
            .current_dir(project_path)
            .output();
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to commit: {stderr}"
        )));
    }

    // Return to original branch
    let output = Command::new("git")
        .args(["checkout", &current_branch])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to return to {current_branch}: {stderr}"
        )));
    }

    Ok(())
}

/// Push centy branch to origin
pub fn push_centy_branch(worktree_path: &Path) -> Result<(), SyncError> {
    let output = Command::new("git")
        .args(["push", "-u", "origin", CENTY_BRANCH])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check for network errors
        if stderr.contains("Could not resolve host")
            || stderr.contains("Connection refused")
            || stderr.contains("Network is unreachable")
        {
            return Err(SyncError::NetworkError(stderr.to_string()));
        }
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to push centy branch: {stderr}"
        )));
    }

    Ok(())
}

/// Pull latest from origin/centy
pub fn pull_centy_branch(worktree_path: &Path) -> Result<PullResult, SyncError> {
    // First fetch
    let output = Command::new("git")
        .args(["fetch", "origin", CENTY_BRANCH])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check for network errors
        if stderr.contains("Could not resolve host")
            || stderr.contains("Connection refused")
            || stderr.contains("Network is unreachable")
        {
            return Err(SyncError::NetworkError(stderr.to_string()));
        }
        // If remote branch doesn't exist yet, that's okay
        if stderr.contains("couldn't find remote ref") {
            return Ok(PullResult::UpToDate);
        }
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to fetch: {stderr}"
        )));
    }

    // Check if we're up to date
    let local_head = get_commit_hash(worktree_path, "HEAD")?;
    let remote_head = match get_commit_hash(worktree_path, &format!("origin/{CENTY_BRANCH}")) {
        Ok(hash) => hash,
        Err(_) => return Ok(PullResult::UpToDate), // Remote doesn't exist yet
    };

    if local_head == remote_head {
        return Ok(PullResult::UpToDate);
    }

    // Try to merge
    let output = Command::new("git")
        .args(["merge", &format!("origin/{CENTY_BRANCH}"), "--no-edit"])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for conflicts
        if stderr.contains("CONFLICT") || stdout.contains("CONFLICT") {
            let conflicts = parse_conflict_files(&stdout, &stderr);
            return Ok(PullResult::Conflict { files: conflicts });
        }

        return Err(SyncError::GitCommandFailed(format!(
            "Failed to merge: {stderr}"
        )));
    }

    // Determine if it was a fast-forward or merge
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Fast-forward") {
        Ok(PullResult::FastForward)
    } else {
        Ok(PullResult::Merged)
    }
}

/// Fetch origin/centy without applying
pub fn fetch_centy_branch(project_path: &Path) -> Result<(), SyncError> {
    let output = Command::new("git")
        .args(["fetch", "origin", CENTY_BRANCH])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("couldn't find remote ref") {
            // Remote branch doesn't exist, that's okay
            return Ok(());
        }
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to fetch: {stderr}"
        )));
    }

    Ok(())
}

/// Create a local tracking branch for the remote centy branch
pub fn create_tracking_branch(project_path: &Path) -> Result<(), SyncError> {
    let output = Command::new("git")
        .args([
            "branch",
            "--track",
            CENTY_BRANCH,
            &format!("origin/{CENTY_BRANCH}"),
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If branch already exists, that's okay
        if stderr.contains("already exists") {
            return Ok(());
        }
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to create tracking branch: {stderr}"
        )));
    }

    Ok(())
}

/// Get the current branch name
fn get_current_branch(project_path: &Path) -> Result<String, SyncError> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(SyncError::GitCommandFailed(
            "Failed to get current branch".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get the commit hash for a ref
fn get_commit_hash(repo_path: &Path, ref_name: &str) -> Result<String, SyncError> {
    let output = Command::new("git")
        .args(["rev-parse", ref_name])
        .current_dir(repo_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(SyncError::GitCommandFailed(format!(
            "Failed to get commit hash for {ref_name}"
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Parse conflict file names from git output
fn parse_conflict_files(stdout: &str, stderr: &str) -> Vec<String> {
    let mut files = Vec::new();
    let combined = format!("{stdout}\n{stderr}");

    for line in combined.lines() {
        if line.contains("CONFLICT") {
            // Extract file name from various conflict message formats
            // "CONFLICT (content): Merge conflict in <file>"
            // "CONFLICT (add/add): Merge conflict in <file>"
            if let Some(pos) = line.find("Merge conflict in ") {
                let file = line[pos + 18..].trim();
                files.push(file.to_string());
            }
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conflict_files() {
        let stdout = "CONFLICT (content): Merge conflict in .centy/issues/abc/issue.md";
        let stderr = "";
        let files = parse_conflict_files(stdout, stderr);
        assert_eq!(files, vec![".centy/issues/abc/issue.md"]);
    }

    #[test]
    fn test_parse_conflict_files_multiple() {
        let stdout = r"CONFLICT (content): Merge conflict in .centy/issues/abc/issue.md
CONFLICT (content): Merge conflict in .centy/issues/def/metadata.json";
        let stderr = "";
        let files = parse_conflict_files(stdout, stderr);
        assert_eq!(
            files,
            vec![
                ".centy/issues/abc/issue.md",
                ".centy/issues/def/metadata.json"
            ]
        );
    }

    #[test]
    fn test_pull_result_variants() {
        assert_eq!(PullResult::UpToDate, PullResult::UpToDate);
        assert_eq!(PullResult::FastForward, PullResult::FastForward);
        assert_eq!(PullResult::Merged, PullResult::Merged);

        let conflict = PullResult::Conflict {
            files: vec!["test.md".to_string()],
        };
        if let PullResult::Conflict { files } = conflict {
            assert_eq!(files, vec!["test.md"]);
        }
    }
}
