//! Built-in editor definitions expressed as EditorConfig structs.
//!
//! These serve as defaults that ship with centy. Users can override
//! them by defining editors with the same ID in `~/.centy/editors.json`.

use super::editor_config::EditorConfig;

/// Returns the built-in editor configurations for VS Code, Terminal, and None.
pub fn builtin_editors() -> Vec<EditorConfig> {
    vec![vscode_editor(), terminal_editor(), none_editor()]
}

/// Built-in VS Code editor config.
///
/// Note: The actual binary detection and launching for VS Code uses the
/// sophisticated platform-specific logic in `vscode.rs` (multi-path fallback).
/// The `detect` and `open_dir` fields are descriptive; the special-case handling
/// in `editor_config.rs` delegates to the native implementation.
fn vscode_editor() -> EditorConfig {
    EditorConfig {
        id: "vscode".to_string(),
        name: "VS Code".to_string(),
        description: "Open workspace in Visual Studio Code".to_string(),
        open_dir: "code --new-window {dir}".to_string(),
        open_file: "code --goto {file}:{line}".to_string(),
        detect: "which code".to_string(),
        terminal_wrapper: false,
        setup_workspace: Some("mkdir -p {dir}/.vscode".to_string()),
    }
}

/// Built-in Terminal editor config.
///
/// Note: The actual terminal detection and launching uses the platform-specific
/// logic in `terminal.rs`. The special-case handling in `editor_config.rs`
/// delegates to the native implementation.
fn terminal_editor() -> EditorConfig {
    EditorConfig {
        id: "terminal".to_string(),
        name: "Terminal".to_string(),
        description: "Open workspace in the OS terminal".to_string(),
        open_dir: String::new(), // Handled by native terminal.rs
        open_file: String::new(),
        detect: String::new(), // Handled by native terminal.rs
        terminal_wrapper: false,
        setup_workspace: None,
    }
}

/// Built-in "none" editor config — don't open any editor.
fn none_editor() -> EditorConfig {
    EditorConfig {
        id: "none".to_string(),
        name: "None".to_string(),
        description: "Don't open any editor".to_string(),
        open_dir: String::new(),
        open_file: String::new(),
        detect: String::new(),
        terminal_wrapper: false,
        setup_workspace: None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_editors_returns_three() {
        let editors = builtin_editors();
        assert_eq!(editors.len(), 3);
    }

    #[test]
    fn test_builtin_editor_ids() {
        let editors = builtin_editors();
        let ids: Vec<&str> = editors.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"vscode"));
        assert!(ids.contains(&"terminal"));
        assert!(ids.contains(&"none"));
    }

    #[test]
    fn test_vscode_editor_fields() {
        let editor = vscode_editor();
        assert_eq!(editor.id, "vscode");
        assert!(!editor.terminal_wrapper);
        assert!(editor.setup_workspace.is_some());
    }

    #[test]
    fn test_terminal_editor_fields() {
        let editor = terminal_editor();
        assert_eq!(editor.id, "terminal");
        assert!(!editor.terminal_wrapper);
        assert!(editor.setup_workspace.is_none());
    }

    #[test]
    fn test_none_editor_fields() {
        let editor = none_editor();
        assert_eq!(editor.id, "none");
        assert!(editor.open_dir.is_empty());
    }
}
