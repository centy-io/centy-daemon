//! CentySyncManager - Main sync orchestrator.
//!
//! This module provides the main interface for syncing `.centy/` data
//! to the centy branch.

use super::branch::{
    centy_branch_exists, create_orphan_centy_branch, create_tracking_branch,
    fetch_centy_branch, pull_centy_branch, push_centy_branch, remote_centy_branch_exists,
    PullResult,
};
use super::worktree::{ensure_sync_worktree, repair_worktree};
use super::{has_remote_origin, is_git_repository, SyncError};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};
use tracing::{debug, info, warn};

/// Global lock map for sync operations by project path
static SYNC_LOCKS: Lazy<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Mode of operation for the sync manager
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncMode {
    /// Full sync with remote (push/pull enabled)
    Full,
    /// Local-only mode (no remote, just local worktree)
    LocalOnly,
    /// Sync disabled (not a git repo or sync explicitly disabled)
    Disabled,
}

/// The main sync manager for a project
#[derive(Debug)]
pub struct CentySyncManager {
    /// Path to the original project
    project_path: PathBuf,
    /// Path to the sync worktree (or project path if disabled)
    sync_worktree: PathBuf,
    /// Current sync mode
    mode: SyncMode,
}

impl CentySyncManager {
    /// Create a new sync manager for a project.
    ///
    /// This will:
    /// 1. Check if the project is a git repository
    /// 2. Check if remote origin exists
    /// 3. Ensure the centy branch and worktree exist
    /// 4. Return appropriate manager based on available features
    pub async fn new(project_path: &Path) -> Result<Self, SyncError> {
        let project_path = project_path.to_path_buf();

        // Check if it's a git repository
        if !is_git_repository(&project_path) {
            debug!("Not a git repository, sync disabled: {:?}", project_path);
            return Ok(Self {
                sync_worktree: project_path.clone(),
                project_path,
                mode: SyncMode::Disabled,
            });
        }

        // Check if remote exists
        let has_remote = has_remote_origin(&project_path)?;
        let mode = if has_remote {
            SyncMode::Full
        } else {
            debug!("No remote origin, using local-only mode: {:?}", project_path);
            SyncMode::LocalOnly
        };

        // Ensure centy branch exists
        if !centy_branch_exists(&project_path)? {
            if mode == SyncMode::Full && remote_centy_branch_exists(&project_path)? {
                // Remote has centy branch, create local tracking branch
                info!("Creating tracking branch for remote centy branch");
                fetch_centy_branch(&project_path)?;
                create_tracking_branch(&project_path)?;
            } else {
                // Create new orphan branch
                info!("Creating new orphan centy branch");
                create_orphan_centy_branch(&project_path)?;
            }
        }

        // Ensure worktree exists
        let sync_worktree = ensure_sync_worktree(&project_path).await?;

        Ok(Self {
            project_path,
            sync_worktree,
            mode,
        })
    }

    /// Create a new sync manager in local-only mode.
    ///
    /// This is used when there's no remote origin configured.
    pub async fn new_local_only(project_path: &Path) -> Result<Self, SyncError> {
        let project_path = project_path.to_path_buf();

        if !is_git_repository(&project_path) {
            return Ok(Self {
                sync_worktree: project_path.clone(),
                project_path,
                mode: SyncMode::Disabled,
            });
        }

        // Ensure centy branch exists
        if !centy_branch_exists(&project_path)? {
            create_orphan_centy_branch(&project_path)?;
        }

        // Ensure worktree exists
        let sync_worktree = ensure_sync_worktree(&project_path).await?;

        Ok(Self {
            project_path,
            sync_worktree,
            mode: SyncMode::LocalOnly,
        })
    }

    /// Get the path to the .centy directory in the sync worktree
    #[must_use]
    pub fn centy_path(&self) -> PathBuf {
        self.sync_worktree.join(".centy")
    }

    /// Get the sync worktree path
    #[must_use]
    pub fn worktree_path(&self) -> &Path {
        &self.sync_worktree
    }

    /// Get the original project path
    #[must_use]
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    /// Get the current sync mode
    #[must_use]
    pub fn mode(&self) -> &SyncMode {
        &self.mode
    }

    /// Check if sync is enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.mode != SyncMode::Disabled
    }

    /// Check if remote sync is available
    #[must_use]
    pub fn has_remote(&self) -> bool {
        self.mode == SyncMode::Full
    }

    /// Acquire lock for this project's sync operations
    async fn acquire_lock(&self) -> OwnedMutexGuard<()> {
        let mut locks = SYNC_LOCKS.lock().await;
        let lock = locks
            .entry(self.project_path.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        drop(locks); // Release the locks map lock before acquiring the project lock
        lock.lock_owned().await
    }

    /// Pull latest changes before a read operation.
    ///
    /// This fetches and merges any changes from the remote centy branch.
    pub async fn pull_before_read(&self) -> Result<(), SyncError> {
        if self.mode != SyncMode::Full {
            return Ok(());
        }

        let _lock = self.acquire_lock().await;

        match pull_centy_branch(&self.sync_worktree) {
            Ok(PullResult::UpToDate) => {
                debug!("Centy branch is up to date");
                Ok(())
            }
            Ok(PullResult::FastForward) => {
                info!("Fast-forwarded centy branch");
                Ok(())
            }
            Ok(PullResult::Merged) => {
                info!("Merged changes from remote centy branch");
                Ok(())
            }
            Ok(PullResult::Conflict { files }) => {
                warn!("Merge conflicts detected in: {:?}", files);
                // Store conflicts for later resolution
                // For now, we'll continue and let the conflict be resolved later
                Ok(())
            }
            Err(SyncError::NetworkError(e)) => {
                warn!("Network error during pull, continuing with local data: {}", e);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Commit and push after a write operation.
    ///
    /// This commits any changes in the sync worktree and pushes to remote.
    pub async fn commit_and_push(&self, message: &str) -> Result<(), SyncError> {
        if self.mode == SyncMode::Disabled {
            return Ok(());
        }

        let _lock = self.acquire_lock().await;

        // Check if there are any changes to commit
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.sync_worktree)
            .output()
            .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

        let status = String::from_utf8_lossy(&status_output.stdout);
        if status.trim().is_empty() {
            debug!("No changes to commit");
            return Ok(());
        }

        // Stage all changes
        let output = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.sync_worktree)
            .output()
            .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SyncError::GitCommandFailed(format!(
                "Failed to stage changes: {stderr}"
            )));
        }

        // Commit
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.sync_worktree)
            .output()
            .map_err(|e| SyncError::GitCommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // "nothing to commit" is not an error
            if !stderr.contains("nothing to commit") {
                return Err(SyncError::GitCommandFailed(format!(
                    "Failed to commit: {stderr}"
                )));
            }
            return Ok(());
        }

        info!("Committed changes: {}", message);

        // Push if in full sync mode
        if self.mode == SyncMode::Full {
            match push_centy_branch(&self.sync_worktree) {
                Ok(()) => {
                    info!("Pushed changes to remote");
                    Ok(())
                }
                Err(SyncError::NetworkError(e)) => {
                    warn!("Network error during push, changes are committed locally: {}", e);
                    // Queue for later push
                    self.queue_pending_push().await?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
    }

    /// Queue a pending push operation for later retry
    async fn queue_pending_push(&self) -> Result<(), SyncError> {
        let queue_path = self.centy_path().join(".sync-pending");
        tokio::fs::write(&queue_path, "pending").await?;
        Ok(())
    }

    /// Check if there are pending push operations
    pub async fn has_pending_push(&self) -> Result<bool, SyncError> {
        let queue_path = self.centy_path().join(".sync-pending");
        Ok(queue_path.exists())
    }

    /// Process any pending push operations
    pub async fn process_pending_push(&self) -> Result<bool, SyncError> {
        if self.mode != SyncMode::Full {
            return Ok(false);
        }

        let queue_path = self.centy_path().join(".sync-pending");
        if !queue_path.exists() {
            return Ok(false);
        }

        match push_centy_branch(&self.sync_worktree) {
            Ok(()) => {
                tokio::fs::remove_file(&queue_path).await?;
                info!("Processed pending push");
                Ok(true)
            }
            Err(SyncError::NetworkError(_)) => {
                // Still offline, keep pending
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Sync a specific file/directory from sync worktree to project.
    ///
    /// This copies files from the sync worktree to the original project.
    pub async fn sync_to_project(&self, relative_path: &Path) -> Result<(), SyncError> {
        if self.mode == SyncMode::Disabled {
            return Ok(());
        }

        let source = self.sync_worktree.join(relative_path);
        let target = self.project_path.join(relative_path);

        if !source.exists() {
            return Ok(());
        }

        // Create parent directories
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Copy file or directory
        if source.is_file() {
            tokio::fs::copy(&source, &target).await?;
        } else if source.is_dir() {
            copy_dir_recursive(&source, &target).await?;
        }

        Ok(())
    }

    /// Sync a specific file/directory from project to sync worktree.
    ///
    /// This copies files from the original project to the sync worktree.
    pub async fn sync_from_project(&self, relative_path: &Path) -> Result<(), SyncError> {
        if self.mode == SyncMode::Disabled {
            return Ok(());
        }

        let source = self.project_path.join(relative_path);
        let target = self.sync_worktree.join(relative_path);

        if !source.exists() {
            return Ok(());
        }

        // Create parent directories
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Copy file or directory
        if source.is_file() {
            tokio::fs::copy(&source, &target).await?;
        } else if source.is_dir() {
            copy_dir_recursive(&source, &target).await?;
        }

        Ok(())
    }

    /// Repair the sync worktree if it's corrupted
    pub async fn repair(&mut self) -> Result<(), SyncError> {
        if self.mode == SyncMode::Disabled {
            return Ok(());
        }

        let new_worktree = repair_worktree(&self.project_path).await?;
        self.sync_worktree = new_worktree;
        Ok(())
    }
}

/// Recursively copy a directory
async fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), SyncError> {
    tokio::fs::create_dir_all(target).await?;

    let mut entries = tokio::fs::read_dir(source).await?;
    while let Some(entry) = entries.next_entry().await? {
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            Box::pin(copy_dir_recursive(&source_path, &target_path)).await?;
        } else {
            tokio::fs::copy(&source_path, &target_path).await?;
        }
    }

    Ok(())
}

/// Initialize sync for a project during centy init.
///
/// This is called during `centy init` to set up the centy branch.
pub async fn initialize_sync(project_path: &Path) -> Result<Option<CentySyncManager>, SyncError> {
    // Check if it's a git repository
    if !is_git_repository(project_path) {
        info!("Not a git repository, skipping sync initialization");
        return Ok(None);
    }

    // Create the sync manager (this will set up everything)
    match CentySyncManager::new(project_path).await {
        Ok(manager) => {
            if manager.is_enabled() {
                info!("Sync initialized for project");
                Ok(Some(manager))
            } else {
                Ok(None)
            }
        }
        Err(e) => {
            warn!("Failed to initialize sync: {}", e);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_mode_equality() {
        assert_eq!(SyncMode::Full, SyncMode::Full);
        assert_eq!(SyncMode::LocalOnly, SyncMode::LocalOnly);
        assert_eq!(SyncMode::Disabled, SyncMode::Disabled);
        assert_ne!(SyncMode::Full, SyncMode::LocalOnly);
    }
}
