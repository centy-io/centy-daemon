//! Type definitions for temporary workspace management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current schema version for workspace registry
pub const CURRENT_WORKSPACE_SCHEMA: u32 = 1;

/// Default TTL for workspaces in hours
pub const DEFAULT_TTL_HOURS: u32 = 12;

/// A single temporary workspace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempWorkspaceEntry {
    /// Path to the source project that was cloned
    pub source_project_path: String,

    /// Issue UUID
    pub issue_id: String,

    /// Human-readable issue display number
    pub issue_display_number: u32,

    /// Issue title for reference
    pub issue_title: String,

    /// Name of the agent configured for this workspace
    pub agent_name: String,

    /// Action type: "plan" or "implement"
    pub action: String,

    /// ISO timestamp when workspace was created
    pub created_at: String,

    /// ISO timestamp when workspace expires
    pub expires_at: String,

    /// Git ref used for the worktree (usually "HEAD")
    pub worktree_ref: String,
}

/// Registry of all temporary workspaces
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRegistry {
    /// Schema version for migrations
    pub schema_version: u32,

    /// ISO timestamp of last update
    pub updated_at: String,

    /// Map of workspace path -> workspace entry
    pub workspaces: HashMap<String, TempWorkspaceEntry>,

    /// Default TTL for new workspaces in hours
    #[serde(default = "default_ttl")]
    pub default_ttl_hours: u32,
}

fn default_ttl() -> u32 {
    DEFAULT_TTL_HOURS
}

impl Default for WorkspaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRegistry {
    /// Create a new empty workspace registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: CURRENT_WORKSPACE_SCHEMA,
            updated_at: crate::utils::now_iso(),
            workspaces: HashMap::new(),
            default_ttl_hours: DEFAULT_TTL_HOURS,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_registry_new() {
        let registry = WorkspaceRegistry::new();
        assert_eq!(registry.schema_version, CURRENT_WORKSPACE_SCHEMA);
        assert!(registry.workspaces.is_empty());
        assert_eq!(registry.default_ttl_hours, DEFAULT_TTL_HOURS);
    }

    #[test]
    fn test_workspace_entry_serialization() {
        let entry = TempWorkspaceEntry {
            source_project_path: "/projects/test".to_string(),
            issue_id: "uuid-1234".to_string(),
            issue_display_number: 42,
            issue_title: "Test Issue".to_string(),
            agent_name: "claude".to_string(),
            action: "plan".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            expires_at: "2025-01-01T12:00:00Z".to_string(),
            worktree_ref: "HEAD".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("sourceProjectPath"));
        assert!(json.contains("issueDisplayNumber"));

        let deserialized: TempWorkspaceEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.issue_id, entry.issue_id);
    }
}
