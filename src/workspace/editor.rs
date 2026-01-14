//! Editor type and launching functionality.
//!
//! Provides the `EditorType` enum and functions for opening workspaces
//! in different editors (VS Code, Terminal, or none).

use super::terminal::open_terminal;
use super::vscode::open_vscode;
use std::path::Path;

/// The editor/environment to open the workspace in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorType {
    /// Open in VS Code (default)
    #[default]
    VSCode,
    /// Open in OS terminal
    Terminal,
    /// Don't open any editor
    None,
}

/// Open the workspace in the specified editor.
///
/// Returns `true` if the editor was successfully opened, `false` otherwise.
pub fn open_editor(editor: EditorType, workspace_path: &Path) -> bool {
    match editor {
        EditorType::VSCode => open_vscode(workspace_path).unwrap_or(false),
        EditorType::Terminal => open_terminal(workspace_path).unwrap_or(false),
        EditorType::None => false,
    }
}
