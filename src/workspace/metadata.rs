//! Centy-specific workspace metadata layer.
//!
//! This module manages workspace metadata that is specific to centy and not
//! managed by gwq, such as issue binding, TTL/expiration, and agent info.
//!
//! Storage: `~/.centy/workspace-metadata.json`

use super::path::calculate_expires_at;
use super::storage::get_centy_config_dir;
use super::types::{TempWorkspaceEntry, DEFAULT_TTL_HOURS};
use super::WorkspaceError;
use crate::utils::now_iso;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::Mutex;

/// Global mutex for metadata file access
static METADATA_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_lock() -> &'static Mutex<()> {
    METADATA_LOCK.get_or_init(|| Mutex::new(()))
}

/// Schema version for workspace metadata
pub const METADATA_SCHEMA_VERSION: u32 = 1;

/// Workspace metadata registry persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataRegistry {
    /// Schema version for migrations
    pub schema_version: u32,

    /// ISO timestamp of last update
    pub updated_at: String,

    /// Map of worktree path -> workspace metadata
    pub workspaces: HashMap<String, WorkspaceMetadata>,

    /// Default TTL for new workspaces in hours
    #[serde(default = "default_ttl")]
    pub default_ttl_hours: u32,
}

fn default_ttl() -> u32 {
    DEFAULT_TTL_HOURS
}

impl Default for MetadataRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: METADATA_SCHEMA_VERSION,
            updated_at: now_iso(),
            workspaces: HashMap::new(),
            default_ttl_hours: DEFAULT_TTL_HOURS,
        }
    }
}

/// Centy-specific metadata stored alongside gwq worktrees
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMetadata {
    /// Worktree path (key)
    pub worktree_path: String,

    /// Path to the source project that was cloned
    pub source_project_path: String,

    /// Issue UUID (empty for standalone workspaces)
    #[serde(default)]
    pub issue_id: String,

    /// Human-readable issue display number (0 for standalone)
    #[serde(default)]
    pub issue_display_number: u32,

    /// Issue title for reference (empty for standalone)
    #[serde(default)]
    pub issue_title: String,

    /// Name of the agent configured for this workspace
    pub agent_name: String,

    /// Action type: "plan", "implement", or "standalone"
    pub action: String,

    /// ISO timestamp when workspace was created
    pub created_at: String,

    /// ISO timestamp when workspace expires
    pub expires_at: String,

    /// Git ref used for the worktree (usually "HEAD")
    #[serde(default = "default_worktree_ref")]
    pub worktree_ref: String,

    /// Whether this is a standalone workspace (not tied to an issue)
    #[serde(default)]
    pub is_standalone: bool,

    /// Unique workspace ID (UUID, generated for standalone workspaces)
    #[serde(default)]
    pub workspace_id: String,

    /// Custom workspace name (for standalone workspaces)
    #[serde(default)]
    pub workspace_name: String,

    /// Custom workspace description/goals (for standalone workspaces)
    #[serde(default)]
    pub workspace_description: String,
}

fn default_worktree_ref() -> String {
    "HEAD".to_string()
}

impl WorkspaceMetadata {
    /// Check if this workspace has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Ok(expires_at) = self.expires_at.parse::<DateTime<Utc>>() {
            expires_at < Utc::now()
        } else {
            // If we can't parse the date, treat as not expired
            false
        }
    }

    /// Extend the TTL of this workspace
    #[allow(dead_code)] // Part of public API
    pub fn extend_ttl(&mut self, hours: u32) {
        self.expires_at = calculate_expires_at(hours);
    }

    /// Convert from `TempWorkspaceEntry` for backwards compatibility
    #[must_use]
    pub fn from_entry(worktree_path: &str, entry: &TempWorkspaceEntry) -> Self {
        Self {
            worktree_path: worktree_path.to_string(),
            source_project_path: entry.source_project_path.clone(),
            issue_id: entry.issue_id.clone(),
            issue_display_number: entry.issue_display_number,
            issue_title: entry.issue_title.clone(),
            agent_name: entry.agent_name.clone(),
            action: entry.action.clone(),
            created_at: entry.created_at.clone(),
            expires_at: entry.expires_at.clone(),
            worktree_ref: entry.worktree_ref.clone(),
            is_standalone: entry.is_standalone,
            workspace_id: entry.workspace_id.clone(),
            workspace_name: entry.workspace_name.clone(),
            workspace_description: entry.workspace_description.clone(),
        }
    }

    /// Convert to `TempWorkspaceEntry` for API compatibility
    #[must_use]
    pub fn to_entry(&self) -> TempWorkspaceEntry {
        TempWorkspaceEntry {
            source_project_path: self.source_project_path.clone(),
            issue_id: self.issue_id.clone(),
            issue_display_number: self.issue_display_number,
            issue_title: self.issue_title.clone(),
            agent_name: self.agent_name.clone(),
            action: self.action.clone(),
            created_at: self.created_at.clone(),
            expires_at: self.expires_at.clone(),
            worktree_ref: self.worktree_ref.clone(),
            is_standalone: self.is_standalone,
            workspace_id: self.workspace_id.clone(),
            workspace_name: self.workspace_name.clone(),
            workspace_description: self.workspace_description.clone(),
        }
    }
}

/// Get the path to the metadata file (~/.centy/workspace-metadata.json)
pub fn get_metadata_path() -> Result<PathBuf, WorkspaceError> {
    Ok(get_centy_config_dir()?.join("workspace-metadata.json"))
}

/// Read the metadata registry from disk
pub async fn read_metadata_registry() -> Result<MetadataRegistry, WorkspaceError> {
    let path = get_metadata_path()?;

    if !path.exists() {
        return Ok(MetadataRegistry::new());
    }

    let content = fs::read_to_string(&path).await?;
    let registry: MetadataRegistry = serde_json::from_str(&content)?;

    Ok(registry)
}

/// Write the metadata registry to disk with locking and atomic write
#[allow(dead_code)] // Part of public API
pub async fn write_metadata_registry(registry: &MetadataRegistry) -> Result<(), WorkspaceError> {
    let _guard = get_lock().lock().await;
    write_metadata_registry_unlocked(registry).await
}

/// Write the registry to disk without acquiring the lock (caller must hold lock)
async fn write_metadata_registry_unlocked(
    registry: &MetadataRegistry,
) -> Result<(), WorkspaceError> {
    let path = get_metadata_path()?;

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

/// Save metadata for a workspace
pub async fn save_metadata(
    worktree_path: &str,
    metadata: WorkspaceMetadata,
) -> Result<(), WorkspaceError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_metadata_registry().await?;
    registry
        .workspaces
        .insert(worktree_path.to_string(), metadata);
    registry.updated_at = now_iso();

    write_metadata_registry_unlocked(&registry).await
}

/// Remove metadata for a workspace
pub async fn remove_metadata(worktree_path: &str) -> Result<bool, WorkspaceError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_metadata_registry().await?;
    let removed = registry.workspaces.remove(worktree_path).is_some();

    if removed {
        registry.updated_at = now_iso();
        write_metadata_registry_unlocked(&registry).await?;
    }

    Ok(removed)
}

/// Get metadata for a specific workspace
pub async fn get_metadata(
    worktree_path: &str,
) -> Result<Option<WorkspaceMetadata>, WorkspaceError> {
    let registry = read_metadata_registry().await?;
    Ok(registry.workspaces.get(worktree_path).cloned())
}

/// Find workspace metadata by issue
pub async fn find_metadata_for_issue(
    source_project_path: &str,
    issue_id: &str,
) -> Result<Option<(String, WorkspaceMetadata)>, WorkspaceError> {
    let registry = read_metadata_registry().await?;

    for (path, metadata) in &registry.workspaces {
        if !metadata.is_standalone
            && metadata.source_project_path == source_project_path
            && metadata.issue_id == issue_id
            && !metadata.is_expired()
        {
            return Ok(Some((path.clone(), metadata.clone())));
        }
    }

    Ok(None)
}

/// Find standalone workspace metadata by ID or name
pub async fn find_standalone_metadata(
    source_project_path: &str,
    workspace_id: Option<&str>,
    workspace_name: Option<&str>,
) -> Result<Option<(String, WorkspaceMetadata)>, WorkspaceError> {
    let registry = read_metadata_registry().await?;

    for (path, metadata) in &registry.workspaces {
        if !metadata.is_standalone || metadata.source_project_path != source_project_path {
            continue;
        }

        // Match by workspace ID if provided
        if let Some(id) = workspace_id {
            if metadata.workspace_id == id && !metadata.is_expired() {
                return Ok(Some((path.clone(), metadata.clone())));
            }
        }

        // Match by workspace name if provided (case-insensitive)
        if let Some(name) = workspace_name {
            if metadata.workspace_name.to_lowercase() == name.to_lowercase()
                && !metadata.is_expired()
            {
                return Ok(Some((path.clone(), metadata.clone())));
            }
        }
    }

    Ok(None)
}

/// Update workspace expiration time
pub async fn update_metadata_expiration(
    worktree_path: &str,
    new_expires_at: &str,
) -> Result<(), WorkspaceError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_metadata_registry().await?;
    if let Some(metadata) = registry.workspaces.get_mut(worktree_path) {
        metadata.expires_at = new_expires_at.to_string();
        registry.updated_at = now_iso();
        write_metadata_registry_unlocked(&registry).await?;
    }

    Ok(())
}

/// List all workspace metadata
pub async fn list_all_metadata() -> Result<Vec<(String, WorkspaceMetadata)>, WorkspaceError> {
    let registry = read_metadata_registry().await?;
    Ok(registry.workspaces.into_iter().collect())
}

/// List workspace metadata with filtering
#[allow(dead_code)] // Part of public API
pub async fn list_metadata(
    include_expired: bool,
    source_project_filter: Option<&str>,
) -> Result<Vec<(String, WorkspaceMetadata, bool)>, WorkspaceError> {
    let registry = read_metadata_registry().await?;

    let mut results: Vec<(String, WorkspaceMetadata, bool)> = registry
        .workspaces
        .into_iter()
        .filter_map(|(path, metadata)| {
            let expired = metadata.is_expired();

            // Filter by expired status
            if !include_expired && expired {
                return None;
            }

            // Filter by source project
            if let Some(filter) = source_project_filter {
                if metadata.source_project_path != filter {
                    return None;
                }
            }

            Some((path, metadata, expired))
        })
        .collect();

    // Sort by created_at descending (newest first)
    results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    Ok(results)
}

/// Get expired workspace metadata
pub async fn get_expired_metadata() -> Result<Vec<(String, WorkspaceMetadata)>, WorkspaceError> {
    let registry = read_metadata_registry().await?;

    Ok(registry
        .workspaces
        .into_iter()
        .filter(|(_, metadata)| metadata.is_expired())
        .collect())
}

/// Get the default TTL from the registry
pub async fn get_default_ttl_from_metadata() -> Result<u32, WorkspaceError> {
    let registry = read_metadata_registry().await?;
    Ok(registry.default_ttl_hours)
}

/// Check if a worktree path exists in the metadata
#[allow(dead_code)] // Part of public API
pub async fn has_metadata(worktree_path: &str) -> Result<bool, WorkspaceError> {
    let registry = read_metadata_registry().await?;
    Ok(registry.workspaces.contains_key(worktree_path))
}

/// Migrate from old workspaces.json to new workspace-metadata.json
#[allow(dead_code)] // Part of migration API
pub async fn migrate_from_old_registry(old_registry_path: &Path) -> Result<u32, WorkspaceError> {
    if !old_registry_path.exists() {
        return Ok(0);
    }

    let content = fs::read_to_string(old_registry_path).await?;

    // Try to parse as the old format
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OldRegistry {
        workspaces: HashMap<String, TempWorkspaceEntry>,
        #[serde(default = "default_ttl")]
        default_ttl_hours: u32,
    }

    let old: OldRegistry = serde_json::from_str(&content)?;
    let mut migrated_count = 0;

    let _guard = get_lock().lock().await;
    let mut registry = read_metadata_registry().await?;

    for (path, entry) in old.workspaces {
        if let std::collections::hash_map::Entry::Vacant(e) = registry.workspaces.entry(path) {
            let metadata = WorkspaceMetadata::from_entry(e.key(), &entry);
            e.insert(metadata);
            migrated_count += 1;
        }
    }

    if migrated_count > 0 {
        registry.default_ttl_hours = old.default_ttl_hours;
        registry.updated_at = now_iso();
        write_metadata_registry_unlocked(&registry).await?;
    }

    Ok(migrated_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_metadata_is_expired() {
        let mut metadata = WorkspaceMetadata {
            worktree_path: "/tmp/test".to_string(),
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

        assert!(metadata.is_expired());

        // Future date
        metadata.expires_at = "2099-01-01T00:00:00Z".to_string();
        assert!(!metadata.is_expired());
    }

    #[test]
    fn test_workspace_metadata_extend_ttl() {
        let mut metadata = WorkspaceMetadata {
            worktree_path: "/tmp/test".to_string(),
            source_project_path: "/test".to_string(),
            issue_id: "uuid".to_string(),
            issue_display_number: 1,
            issue_title: "Test".to_string(),
            agent_name: "claude".to_string(),
            action: "plan".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            expires_at: "2020-01-01T00:00:00Z".to_string(),
            worktree_ref: "HEAD".to_string(),
            is_standalone: false,
            workspace_id: String::new(),
            workspace_name: String::new(),
            workspace_description: String::new(),
        };

        assert!(metadata.is_expired());
        metadata.extend_ttl(12);
        assert!(!metadata.is_expired());
    }

    #[test]
    fn test_metadata_registry_new() {
        let registry = MetadataRegistry::new();
        assert_eq!(registry.schema_version, METADATA_SCHEMA_VERSION);
        assert!(registry.workspaces.is_empty());
        assert_eq!(registry.default_ttl_hours, DEFAULT_TTL_HOURS);
    }

    #[test]
    fn test_workspace_metadata_to_entry_roundtrip() {
        let metadata = WorkspaceMetadata {
            worktree_path: "/tmp/test".to_string(),
            source_project_path: "/projects/test".to_string(),
            issue_id: "uuid-1234".to_string(),
            issue_display_number: 42,
            issue_title: "Test Issue".to_string(),
            agent_name: "claude".to_string(),
            action: "plan".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            expires_at: "2025-01-01T12:00:00Z".to_string(),
            worktree_ref: "HEAD".to_string(),
            is_standalone: false,
            workspace_id: String::new(),
            workspace_name: String::new(),
            workspace_description: String::new(),
        };

        let entry = metadata.to_entry();
        let roundtrip = WorkspaceMetadata::from_entry(&metadata.worktree_path, &entry);

        assert_eq!(roundtrip.source_project_path, metadata.source_project_path);
        assert_eq!(roundtrip.issue_id, metadata.issue_id);
        assert_eq!(
            roundtrip.issue_display_number,
            metadata.issue_display_number
        );
    }
}
