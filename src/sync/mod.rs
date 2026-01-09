//! Centy branch synchronization module.
//!
//! This module provides automatic synchronization of `.centy/` data via a dedicated
//! orphan `centy` branch. This enables:
//! - Multi-machine sync of issues, docs, and PRs
//! - Cross-branch consistency (issues visible on all branches)
//! - Remote backup without polluting code branches
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Daemon (gRPC)                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │   CreateIssue  GetIssue  UpdateIssue  ...                  │
//! │       │           │          │                              │
//! │       └───────────┴──────────┴──────────────────┐          │
//! │                              ┌───────────────────▼────────┐ │
//! │                              │     CentySyncManager       │ │
//! │                              │  • pull_before_read()      │ │
//! │                              │  • commit_and_push()       │ │
//! │                              │  • merge_conflicts()       │ │
//! │                              └───────────────────┬────────┘ │
//! │                              ┌───────────────────▼────────┐ │
//! │                              │   Centy Branch Worktree    │ │
//! │                              │  ~/.centy/sync/{hash}/     │ │
//! │                              │      └── .centy/           │ │
//! │                              └───────────────────┬────────┘ │
//! │                                         │ git push/pull     │
//! │                              ┌──────────▼─────────────────┐ │
//! │                              │    Remote: origin/centy    │ │
//! │                              └────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod branch;
pub mod conflicts;
pub mod manager;
pub mod merge;
pub mod sync_aware;
pub mod worktree;

// Re-exports for library API (not all used by the binary)
#[allow(unused_imports)]
pub use branch::{
    centy_branch_exists, create_orphan_centy_branch, fetch_centy_branch, pull_centy_branch,
    push_centy_branch, remote_centy_branch_exists, PullResult,
};
#[allow(unused_imports)]
pub use conflicts::{
    get_conflict, list_conflicts, resolve_conflict, store_conflict, ConflictInfo,
    ConflictResolution,
};
pub use manager::CentySyncManager;
#[allow(unused_imports)]
pub use merge::{merge_json_metadata, merge_markdown, MergeResult};
#[allow(unused_imports)]
pub use worktree::{
    ensure_sync_worktree, get_sync_centy_path, get_sync_worktree_path, remove_sync_worktree,
    sync_worktree_exists,
};

use std::path::PathBuf;
use thiserror::Error;

/// The name of the centy sync branch
pub const CENTY_BRANCH: &str = "centy";

/// Error types for sync operations
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Not a git repository")]
    NotGitRepository,

    #[error("No remote 'origin' configured")]
    NoRemote,

    #[error("Worktree error: {0}")]
    WorktreeError(String),

    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Merge conflict in {file}")]
    MergeConflict {
        file: String,
        conflict_path: PathBuf,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("Sync is disabled for this project (local-only mode)")]
    SyncDisabled,

    #[error("Conflict not found: {0}")]
    ConflictNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Check if a project has a remote origin configured
pub fn has_remote_origin(project_path: &std::path::Path) -> Result<bool, SyncError> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(project_path)
        .output()
        .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

    Ok(output.status.success())
}

/// Check if the current directory is a git repository
pub fn is_git_repository(project_path: &std::path::Path) -> bool {
    use std::process::Command;

    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_centy_branch_constant() {
        assert_eq!(CENTY_BRANCH, "centy");
    }

    #[test]
    fn test_is_git_repository_non_git() {
        // /tmp is typically not a git repository
        let non_git = Path::new("/tmp");
        assert!(!is_git_repository(non_git));
    }
}
