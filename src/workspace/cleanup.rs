//! Workspace cleanup logic using gwq.
//!
//! Handles removing temporary workspaces and cleaning up git worktrees.

use super::gwq_client::GwqClient;
use super::metadata::{get_expired_metadata, get_metadata, remove_metadata};
use super::storage::remove_workspace;
use super::WorkspaceError;
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

/// Get the gwq client for cleanup operations
fn get_gwq_client() -> Result<GwqClient, WorkspaceError> {
    GwqClient::new().map_err(|e| WorkspaceError::GitError(e.to_string()))
}

/// Clean up a single workspace.
///
/// This function:
/// 1. Removes the git worktree via gwq
/// 2. Removes the workspace directory (if it still exists)
/// 3. Removes the entry from metadata and legacy registry
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

    // Get the workspace metadata to find the source project
    let metadata = get_metadata(workspace_path).await?;

    // Fall back to legacy storage if metadata not found
    let entry = if metadata.is_none() {
        super::storage::get_workspace(workspace_path).await?
    } else {
        None
    };

    let workspace_path_buf = Path::new(workspace_path);

    // Get gwq client
    let gwq = match get_gwq_client() {
        Ok(client) => client,
        Err(e) => {
            let error_msg = format!("Failed to get gwq client: {e}");
            warn!("{error_msg}");
            if !force {
                result.error = Some(error_msg);
                return Ok(result);
            }
            // Continue with directory cleanup only
            return cleanup_directory_only(workspace_path, result, force).await;
        }
    };

    // Try to remove the git worktree
    let source_project = metadata
        .as_ref()
        .map(|m| m.source_project_path.as_str())
        .or_else(|| entry.as_ref().map(|e| e.source_project_path.as_str()));

    if let Some(src) = source_project {
        let source_path = Path::new(src);
        if source_path.exists() {
            match gwq.remove_worktree_from_repo(source_path, workspace_path_buf, true) {
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
            if let Err(e) = gwq.prune(source_path) {
                warn!("Failed to prune worktrees: {e}");
            }
        } else {
            warn!("Source project no longer exists: {src}");
        }
    } else {
        match gwq.remove_worktree(workspace_path_buf, true) {
            Ok(()) => {
                result.worktree_removed = true;
                info!("Removed git worktree (direct): {workspace_path}");
            }
            Err(e) => {
                warn!("Failed to remove worktree: {e}");
            }
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

    // Remove from metadata registry
    let _ = remove_metadata(workspace_path).await;

    // Remove from legacy registry
    let _ = remove_workspace(workspace_path).await;

    Ok(result)
}

/// Clean up directory only (when gwq is not available)
async fn cleanup_directory_only(
    workspace_path: &str,
    mut result: CleanupResult,
    force: bool,
) -> Result<CleanupResult, WorkspaceError> {
    let workspace_path_buf = Path::new(workspace_path);

    // Remove the workspace directory if it exists
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
        result.directory_removed = true;
    }

    // Remove from metadata registry
    let _ = remove_metadata(workspace_path).await;

    // Remove from legacy registry
    let _ = remove_workspace(workspace_path).await;

    Ok(result)
}

/// Clean up all expired workspaces.
///
/// Returns a list of cleanup results for each workspace processed.
pub async fn cleanup_expired_workspaces() -> Result<Vec<CleanupResult>, WorkspaceError> {
    let mut results = Vec::new();

    // Get all expired workspaces from metadata
    let expired_metadata = get_expired_metadata().await?;

    // Also check legacy storage for any workspaces not in metadata
    let legacy_registry = super::storage::read_registry().await?;
    let expired_legacy: Vec<String> = legacy_registry
        .workspaces
        .iter()
        .filter(|(_, entry)| super::storage::is_expired(entry))
        .map(|(path, _)| path.clone())
        .collect();

    // Combine and deduplicate
    let mut all_expired: std::collections::HashSet<String> =
        expired_metadata.into_iter().map(|(path, _)| path).collect();
    for path in expired_legacy {
        all_expired.insert(path);
    }

    info!("Found {} expired workspaces to clean up", all_expired.len());

    for path in all_expired {
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

    // Run gwq prune for any orphaned worktrees (best effort)
    if let Ok(gwq) = get_gwq_client() {
        // Get unique source projects from all metadata
        if let Ok(all_metadata) = super::metadata::list_all_metadata().await {
            let source_projects: std::collections::HashSet<String> = all_metadata
                .iter()
                .map(|(_, meta)| meta.source_project_path.clone())
                .collect();

            for source_path in source_projects {
                let path = Path::new(&source_path);
                if path.exists() {
                    if let Err(e) = gwq.prune(path) {
                        warn!("Failed to prune worktrees in {source_path}: {e}");
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Count successfully cleaned workspaces
#[allow(dead_code)] // Utility for workspace cleanup reporting
pub fn count_cleaned(results: &[CleanupResult]) -> u32 {
    results.iter().filter(|r| r.error.is_none()).count() as u32
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
