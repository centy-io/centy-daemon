//! Editor type and launching functionality.
//!
//! This module bridges the legacy `EditorType` enum with the new config-driven
//! editor system in `editor_config`. The `EditorType` enum is kept for backward
//! compatibility with existing code paths; new code should use editor IDs directly.

use super::editor_config::{find_editor, open_editor_by_config, run_editor_setup};
use std::path::Path;

/// The editor/environment to open the workspace in.
///
/// **Deprecated**: Prefer using string-based editor IDs with `open_editor_by_id` instead.
/// This enum is kept for backward compatibility during the transition period.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EditorType {
    /// Open in VS Code (default)
    #[default]
    VSCode,
    /// Open in OS terminal
    Terminal,
    /// Don't open any editor
    None,
    /// A custom editor, identified by its config ID
    Custom(String),
}

impl EditorType {
    /// Convert an editor ID string to an EditorType.
    #[must_use]
    pub fn from_id(id: &str) -> Self {
        match id {
            "vscode" => Self::VSCode,
            "terminal" => Self::Terminal,
            "none" | "" => Self::None,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Convert this EditorType to an editor ID string.
    #[must_use]
    pub fn to_id(&self) -> &str {
        match self {
            Self::VSCode => "vscode",
            Self::Terminal => "terminal",
            Self::None => "none",
            Self::Custom(id) => id,
        }
    }
}

/// Open the workspace in the specified editor.
///
/// Returns `true` if the editor was successfully opened, `false` otherwise.
/// This function supports both built-in editors and custom editors via the
/// config-driven system.
pub fn open_editor(editor: EditorType, workspace_path: &Path) -> bool {
    let editor_id = editor.to_id();
    open_editor_by_id(editor_id, workspace_path)
}

/// Open the workspace using an editor ID string.
///
/// Looks up the editor config by ID and launches it. Falls back to built-in
/// behavior for "vscode" and "terminal".
pub fn open_editor_by_id(editor_id: &str, workspace_path: &Path) -> bool {
    if editor_id == "none" || editor_id.is_empty() {
        return false;
    }

    // Use a blocking runtime to call async find_editor from sync context
    let editor_config = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(find_editor(editor_id))
    });

    match editor_config {
        Some(config) => open_editor_by_config(&config, workspace_path),
        None => {
            tracing::warn!("Editor '{}' not found in config, cannot open", editor_id);
            false
        }
    }
}

/// Run editor-specific workspace setup by editor ID.
pub async fn run_editor_setup_by_id(editor_id: &str, workspace_path: &Path) {
    if editor_id == "none" || editor_id.is_empty() {
        return;
    }

    if let Some(config) = find_editor(editor_id).await {
        run_editor_setup(&config, workspace_path).await;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_type_from_id() {
        assert_eq!(EditorType::from_id("vscode"), EditorType::VSCode);
        assert_eq!(EditorType::from_id("terminal"), EditorType::Terminal);
        assert_eq!(EditorType::from_id("none"), EditorType::None);
        assert_eq!(EditorType::from_id(""), EditorType::None);
        assert_eq!(
            EditorType::from_id("zed"),
            EditorType::Custom("zed".to_string())
        );
    }

    #[test]
    fn test_editor_type_to_id() {
        assert_eq!(EditorType::VSCode.to_id(), "vscode");
        assert_eq!(EditorType::Terminal.to_id(), "terminal");
        assert_eq!(EditorType::None.to_id(), "none");
        assert_eq!(EditorType::Custom("zed".to_string()).to_id(), "zed");
    }

    #[test]
    fn test_editor_type_roundtrip() {
        let ids = ["vscode", "terminal", "none", "zed", "neovim"];
        for id in ids {
            let editor_type = EditorType::from_id(id);
            assert_eq!(editor_type.to_id(), id);
        }
    }
}
