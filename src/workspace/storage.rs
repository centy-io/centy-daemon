//! Storage operations for workspace registry.
//!
//! Manages persistence of workspace tracking data to ~/.centy/workspaces.json

use super::types::{TempWorkspaceEntry, WorkspaceRegistry};
use super::WorkspaceError;
use crate::utils::now_iso;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::Mutex;

/// Global mutex for workspace registry file access
static WORKSPACE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_lock() -> &'static Mutex<()> {
    WORKSPACE_LOCK.get_or_init(|| Mutex::new(()))
}

/// Get the path to the global centy config directory (~/.centy)
pub fn get_centy_config_dir() -> Result<PathBuf, WorkspaceError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| WorkspaceError::HomeDirNotFound)?;

    Ok(PathBuf::from(home).join(".centy"))
}

/// Get the path to the workspace registry file (~/.centy/workspaces.json)
pub fn get_workspaces_path() -> Result<PathBuf, WorkspaceError> {
    Ok(get_centy_config_dir()?.join("workspaces.json"))
}

/// Read the workspace registry from disk
pub async fn read_registry() -> Result<WorkspaceRegistry, WorkspaceError> {
    let path = get_workspaces_path()?;

    if !path.exists() {
        return Ok(WorkspaceRegistry::new());
    }

    let content = fs::read_to_string(&path).await?;
    let registry: WorkspaceRegistry = serde_json::from_str(&content)?;

    Ok(registry)
}

/// Write the workspace registry to disk with locking and atomic write
#[allow(dead_code)] // Part of workspace management infrastructure
pub async fn write_registry(registry: &WorkspaceRegistry) -> Result<(), WorkspaceError> {
    let _guard = get_lock().lock().await;
    write_registry_unlocked(registry).await
}

/// Write the registry to disk without acquiring the lock (caller must hold lock)
async fn write_registry_unlocked(registry: &WorkspaceRegistry) -> Result<(), WorkspaceError> {
    let path = get_workspaces_path()?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Write atomically using temp file + rename
    let temp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(registry)?;
    fs::write(&temp_path, &content).await?;
    fs::rename(&temp_path, &path).await?;

    Ok(())
}

/// Add a workspace entry to the registry
pub async fn add_workspace(
    workspace_path: &str,
    entry: TempWorkspaceEntry,
) -> Result<(), WorkspaceError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;
    registry
        .workspaces
        .insert(workspace_path.to_string(), entry);
    registry.updated_at = now_iso();

    write_registry_unlocked(&registry).await
}

/// Remove a workspace entry from the registry
pub async fn remove_workspace(workspace_path: &str) -> Result<bool, WorkspaceError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;
    let removed = registry.workspaces.remove(workspace_path).is_some();

    if removed {
        registry.updated_at = now_iso();
        write_registry_unlocked(&registry).await?;
    }

    Ok(removed)
}

/// Get a workspace entry by path
pub async fn get_workspace(
    workspace_path: &str,
) -> Result<Option<TempWorkspaceEntry>, WorkspaceError> {
    let registry = read_registry().await?;
    Ok(registry.workspaces.get(workspace_path).cloned())
}

/// Find an existing workspace for a specific issue in a source project.
///
/// Returns the workspace path and entry if found and not expired.
#[allow(dead_code)] // Part of public API, kept for backwards compatibility
pub async fn find_workspace_for_issue(
    source_project_path: &str,
    issue_id: &str,
) -> Result<Option<(String, TempWorkspaceEntry)>, WorkspaceError> {
    let registry = read_registry().await?;

    for (path, entry) in &registry.workspaces {
        // Match by source project and issue ID (skip standalone workspaces)
        if !entry.is_standalone
            && entry.source_project_path == source_project_path
            && entry.issue_id == issue_id
        {
            // Check if not expired
            if !is_expired(entry) {
                return Ok(Some((path.clone(), entry.clone())));
            }
        }
    }

    Ok(None)
}

/// Find an existing standalone workspace by workspace ID or name.
///
/// Returns the workspace path and entry if found and not expired.
#[allow(dead_code)] // Part of public API, kept for backwards compatibility
pub async fn find_standalone_workspace(
    source_project_path: &str,
    workspace_id: Option<&str>,
    workspace_name: Option<&str>,
) -> Result<Option<(String, TempWorkspaceEntry)>, WorkspaceError> {
    let registry = read_registry().await?;

    for (path, entry) in &registry.workspaces {
        // Only look at standalone workspaces for this project
        if !entry.is_standalone || entry.source_project_path != source_project_path {
            continue;
        }

        // Match by workspace ID if provided
        if let Some(id) = workspace_id {
            if entry.workspace_id == id && !is_expired(entry) {
                return Ok(Some((path.clone(), entry.clone())));
            }
        }

        // Match by workspace name if provided (case-insensitive)
        if let Some(name) = workspace_name {
            if entry.workspace_name.to_lowercase() == name.to_lowercase() && !is_expired(entry) {
                return Ok(Some((path.clone(), entry.clone())));
            }
        }
    }

    Ok(None)
}

/// Update an existing workspace entry's expiration time.
///
/// This is used when reopening an existing workspace to extend its TTL.
pub async fn update_workspace_expiration(
    workspace_path: &str,
    new_expires_at: &str,
) -> Result<(), WorkspaceError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;
    if let Some(entry) = registry.workspaces.get_mut(workspace_path) {
        entry.expires_at = new_expires_at.to_string();
        registry.updated_at = now_iso();
        write_registry_unlocked(&registry).await?;
    }

    Ok(())
}

/// Check if a workspace has expired
pub fn is_expired(entry: &TempWorkspaceEntry) -> bool {
    if let Ok(expires_at) = entry.expires_at.parse::<DateTime<Utc>>() {
        expires_at < Utc::now()
    } else {
        // If we can't parse the date, treat as not expired
        false
    }
}

/// List all workspaces with optional filtering
///
/// Returns tuples of (path, entry, is_expired)
pub async fn list_workspaces(
    include_expired: bool,
    source_project_filter: Option<&str>,
) -> Result<Vec<(String, TempWorkspaceEntry, bool)>, WorkspaceError> {
    let registry = read_registry().await?;

    let mut results: Vec<(String, TempWorkspaceEntry, bool)> = registry
        .workspaces
        .into_iter()
        .filter_map(|(path, entry)| {
            let expired = is_expired(&entry);

            // Filter by expired status
            if !include_expired && expired {
                return None;
            }

            // Filter by source project
            if let Some(filter) = source_project_filter {
                if entry.source_project_path != filter {
                    return None;
                }
            }

            Some((path, entry, expired))
        })
        .collect();

    // Sort by created_at descending (newest first)
    results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    Ok(results)
}

/// Count expired workspaces
#[allow(dead_code)] // Part of workspace management infrastructure
pub async fn count_expired() -> Result<u32, WorkspaceError> {
    let registry = read_registry().await?;

    let count = registry
        .workspaces
        .values()
        .filter(|entry| is_expired(entry))
        .count();

    Ok(count as u32)
}

/// Get the default TTL from the registry
#[allow(dead_code)] // Part of public API, kept for backwards compatibility
pub async fn get_default_ttl() -> Result<u32, WorkspaceError> {
    let registry = read_registry().await?;
    Ok(registry.default_ttl_hours)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_workspaces_path() {
        // This test will work if HOME or USERPROFILE is set
        let result = get_workspaces_path();
        if std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok() {
            assert!(result.is_ok());
            let path = result.unwrap();
            assert!(path.ends_with("workspaces.json"));
            assert!(path.to_string_lossy().contains(".centy"));
        }
    }

    #[test]
    fn test_is_expired() {
        let mut entry = TempWorkspaceEntry {
            source_project_path: "/test".to_string(),
            issue_id: "uuid".to_string(),
            issue_display_number: 1,
            issue_title: "Test".to_string(),
            agent_name: "claude".to_string(),
            action: "plan".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            expires_at: "2020-01-01T00:00:00Z".to_string(), // Past date
            worktree_ref: "HEAD".to_string(),
            is_standalone: false,
            workspace_id: String::new(),
            workspace_name: String::new(),
            workspace_description: String::new(),
        };

        assert!(is_expired(&entry));

        // Future date
        entry.expires_at = "2099-01-01T00:00:00Z".to_string();
        assert!(!is_expired(&entry));
    }
}
