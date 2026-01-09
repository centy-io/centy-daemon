//! Workspace cleanup logic.
//!
//! Handles removing temporary workspaces and cleaning up git worktrees.

use super::storage::{is_expired, read_registry, remove_workspace};
use super::WorkspaceError;
use crate::pr::git::{prune_worktrees, remove_worktree};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

/// Result of cleaning up a single workspace
#[derive(Debug, Clone)]
pub struct CleanupResult {
    /// Path to the workspace
    pub workspace_path: String,

    /// Whether the git worktree was removed
    pub worktree_removed: bool,

    /// Whether the directory was removed
    pub directory_removed: bool,

    /// Error message if cleanup failed
    pub error: Option<String>,
}

/// Clean up a single workspace.
///
/// This function:
/// 1. Removes the git worktree from the source project
/// 2. Removes the workspace directory (if it still exists)
/// 3. Removes the entry from the registry
///
/// # Arguments
/// * `workspace_path` - Path to the workspace to clean up
/// * `force` - If true, force removal even if worktree removal fails
pub async fn cleanup_workspace(
    workspace_path: &str,
    force: bool,
) -> Result<CleanupResult, WorkspaceError> {
    let mut result = CleanupResult {
        workspace_path: workspace_path.to_string(),
        worktree_removed: false,
        directory_removed: false,
        error: None,
    };

    // Get the workspace entry to find the source project
    let entry = super::storage::get_workspace(workspace_path).await?;

    let workspace_path_buf = Path::new(workspace_path);

    if let Some(entry) = entry {
        let source_path = Path::new(&entry.source_project_path);

        // Try to remove the git worktree
        if source_path.exists() {
            match remove_worktree(source_path, workspace_path_buf) {
                Ok(()) => {
                    result.worktree_removed = true;
                    info!("Removed git worktree: {workspace_path}");
                }
                Err(e) => {
                    let error_msg = format!("Failed to remove worktree: {e}");
                    warn!("{error_msg}");
                    if !force {
                        result.error = Some(error_msg);
                        return Ok(result);
                    }
                }
            }

            // Prune stale worktree references
            if let Err(e) = prune_worktrees(source_path) {
                warn!("Failed to prune worktrees: {e}");
            }
        } else {
            // Source project doesn't exist, worktree is orphaned
            warn!(
                "Source project no longer exists: {}",
                entry.source_project_path
            );
        }
    }

    // Remove the workspace directory if it still exists
    if workspace_path_buf.exists() {
        match fs::remove_dir_all(workspace_path_buf).await {
            Ok(()) => {
                result.directory_removed = true;
                info!("Removed workspace directory: {workspace_path}");
            }
            Err(e) => {
                let error_msg = format!("Failed to remove directory: {e}");
                warn!("{error_msg}");
                if !force {
                    result.error = Some(error_msg);
                    return Ok(result);
                }
            }
        }
    } else {
        // Directory doesn't exist, that's fine
        result.directory_removed = true;
    }

    // Remove from registry
    remove_workspace(workspace_path).await?;

    Ok(result)
}

/// Clean up all expired workspaces.
///
/// Returns a list of cleanup results for each workspace processed.
pub async fn cleanup_expired_workspaces() -> Result<Vec<CleanupResult>, WorkspaceError> {
    let mut results = Vec::new();

    // Get all workspaces including expired ones
    let registry = read_registry().await?;

    let expired_paths: Vec<String> = registry
        .workspaces
        .iter()
        .filter(|(_, entry)| is_expired(entry))
        .map(|(path, _)| path.clone())
        .collect();

    info!("Found {} expired workspaces to clean up", expired_paths.len());

    for path in expired_paths {
        let result = cleanup_workspace(&path, true).await;
        match result {
            Ok(r) => results.push(r),
            Err(e) => {
                warn!("Error cleaning up workspace {path}: {e}");
                results.push(CleanupResult {
                    workspace_path: path,
                    worktree_removed: false,
                    directory_removed: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

/// Count successfully cleaned workspaces
#[allow(dead_code)] // Utility for workspace cleanup reporting
pub fn count_cleaned(results: &[CleanupResult]) -> u32 {
    results
        .iter()
        .filter(|r| r.error.is_none())
        .count() as u32
}

/// Get paths that failed to clean
#[allow(dead_code)] // Utility for workspace cleanup reporting
pub fn get_failed_paths(results: &[CleanupResult]) -> Vec<String> {
    results
        .iter()
        .filter(|r| r.error.is_some())
        .map(|r| r.workspace_path.clone())
        .collect()
}

/// Get paths that were successfully cleaned
#[allow(dead_code)] // Utility for workspace cleanup reporting
pub fn get_cleaned_paths(results: &[CleanupResult]) -> Vec<String> {
    results
        .iter()
        .filter(|r| r.error.is_none())
        .map(|r| r.workspace_path.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_result() {
        let result = CleanupResult {
            workspace_path: "/tmp/test".to_string(),
            worktree_removed: true,
            directory_removed: true,
            error: None,
        };

        assert!(result.error.is_none());
    }

    #[test]
    fn test_count_cleaned() {
        let results = vec![
            CleanupResult {
                workspace_path: "/tmp/a".to_string(),
                worktree_removed: true,
                directory_removed: true,
                error: None,
            },
            CleanupResult {
                workspace_path: "/tmp/b".to_string(),
                worktree_removed: false,
                directory_removed: false,
                error: Some("failed".to_string()),
            },
            CleanupResult {
                workspace_path: "/tmp/c".to_string(),
                worktree_removed: true,
                directory_removed: true,
                error: None,
            },
        ];

        assert_eq!(count_cleaned(&results), 2);
    }

    #[test]
    fn test_get_failed_paths() {
        let results = vec![
            CleanupResult {
                workspace_path: "/tmp/a".to_string(),
                worktree_removed: true,
                directory_removed: true,
                error: None,
            },
            CleanupResult {
                workspace_path: "/tmp/b".to_string(),
                worktree_removed: false,
                directory_removed: false,
                error: Some("failed".to_string()),
            },
        ];

        let failed = get_failed_paths(&results);
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0], "/tmp/b");
    }
}
