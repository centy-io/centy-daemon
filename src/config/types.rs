use crate::hooks::HookDefinition;
use crate::link::CustomLinkTypeDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Default priority levels (3 = high/medium/low)
pub fn default_priority_levels() -> u32 {
    3
}
/// Default allowed states for issues (used only during migration to per-item-type config.yaml)
pub fn default_allowed_states() -> Vec<String> {
    vec![
        "open".to_string(),
        "planning".to_string(),
        "in-progress".to_string(),
        "closed".to_string(),
    ]
}
/// Workspace configuration section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfig {
    /// Whether opening a temp workspace automatically updates the issue status to "in-progress".
    /// `None` means the user hasn't configured it yet (server returns `requires_status_config`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_status_on_open: Option<bool>,
}
/// Centy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CentyConfig {
    /// Project version (semver string). Defaults to daemon version if not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Number of priority levels (1-10). Default is 3 (high/medium/low).
    #[serde(default = "default_priority_levels")]
    pub priority_levels: u32,
    #[serde(default)]
    pub custom_fields: Vec<mdstore::CustomFieldDef>,
    #[serde(default)]
    pub defaults: HashMap<String, String>,
    /// Legacy: allowed status values. Migrated to per-item-type `config.yaml`.
    #[serde(default = "default_allowed_states", skip_serializing)]
    pub allowed_states: Vec<String>,
    /// State colors: state name → hex color (e.g., "open" → "#10b981")
    #[serde(default)]
    pub state_colors: HashMap<String, String>,
    /// Priority colors: priority level → hex color (e.g., "1" → "#ef4444")
    #[serde(default)]
    pub priority_colors: HashMap<String, String>,
    /// Custom link types (in addition to built-in types)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_link_types: Vec<CustomLinkTypeDefinition>,
    /// Default editor ID for this project (e.g., "vscode", "terminal", "zed").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_editor: Option<String>,
    /// Lifecycle hooks (bash scripts to run before/after operations)
    #[serde(default)]
    pub hooks: Vec<HookDefinition>,
    /// Workspace settings (e.g. auto-update issue status on open)
    #[serde(default)]
    pub workspace: WorkspaceConfig,
}
impl CentyConfig {
    /// Get the effective version (config version or daemon default).
    #[must_use]
    pub fn effective_version(&self) -> String {
        self.version
            .clone()
            .unwrap_or_else(|| crate::utils::CENTY_VERSION.to_string())
    }
}
impl Default for CentyConfig {
    fn default() -> Self {
        Self {
            version: None,
            priority_levels: default_priority_levels(),
            custom_fields: Vec::new(),
            defaults: HashMap::new(),
            allowed_states: default_allowed_states(),
            state_colors: HashMap::new(),
            priority_colors: HashMap::new(),
            custom_link_types: Vec::new(),
            default_editor: None,
            hooks: Vec::new(),
            workspace: WorkspaceConfig::default(),
        }
    }
}
/// Project metadata stored in .centy/project.json (version-controlled, shared with team)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    /// Project-scope custom title (visible to all users)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
