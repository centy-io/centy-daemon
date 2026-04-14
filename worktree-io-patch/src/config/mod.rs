mod ops;
mod ops_get_set;
mod ser;

#[cfg(test)]
#[path = "ops_tests.rs"]
mod ops_tests;

#[cfg(test)]
#[path = "ops_auto_prune_tests.rs"]
mod ops_auto_prune_tests;

#[cfg(test)]
#[path = "ops_temp_tests.rs"]
mod ops_temp_tests;
use serde::{Deserialize, Serialize};

use crate::ttl::Ttl;

/// Top-level configuration for the worktree CLI.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Editor configuration.
    pub editor: EditorConfig,
    /// Workspace open behavior.
    pub open: OpenConfig,
    /// Hook scripts run around the open command.
    pub hooks: HooksConfig,
    /// Workspace lifecycle configuration.
    pub workspace: WorkspaceConfig,
}

/// Workspace lifecycle configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WorkspaceConfig {
    /// Maximum age of a workspace before it is considered expired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<Ttl>,
    /// When true, expired worktrees are pruned each time `open` is invoked.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub auto_prune: bool,
    /// When true, worktrees are stored under the OS temp directory.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub temp: bool,
}

/// Shell scripts executed before and after opening a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksConfig {
    /// Script run before opening the workspace.
    #[serde(rename = "pre:open", skip_serializing_if = "Option::is_none", default)]
    pub pre_open: Option<String>,
    /// Script run after opening the workspace.
    #[serde(rename = "post:open", skip_serializing_if = "Option::is_none", default)]
    pub post_open: Option<String>,
}

/// Editor-related configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EditorConfig {
    /// Command to launch the editor, e.g. "code ." or "nvim ."
    pub command: Option<String>,
    /// When true, the editor opens in the background (fire-and-forget).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub background: bool,
}

/// Controls how the workspace is opened.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenConfig {
    /// Whether to launch the configured editor when opening a workspace.
    pub editor: bool,
}

impl Default for OpenConfig {
    fn default() -> Self {
        Self { editor: true }
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
