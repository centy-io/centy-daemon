//! Configurable editor support.
//!
//! Provides an editor configuration system where users can define custom editors
//! via user-level config (`~/.centy/editors.json`). Built-in editors (VS Code, Terminal)
//! are expressed as default configs and can be overridden.

use super::terminal::open_platform_terminal;
use super::vscode::{find_vscode_binary, setup_vscode_config};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tokio::fs;
use tracing::warn;

/// A user-defined or built-in editor configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorConfig {
    /// Unique identifier (e.g., "vscode", "zed", "neovim")
    pub id: String,

    /// Human-readable display name
    pub name: String,

    /// Brief description of the editor
    #[serde(default)]
    pub description: String,

    /// Command template to open a project directory.
    /// Supports `{dir}` placeholder. Example: `"code --new-window {dir}"`
    pub open_dir: String,

    /// Command template to open a file at a line number.
    /// Supports `{file}` and `{line}` placeholders. Example: `"code --goto {file}:{line}"`
    #[serde(default)]
    pub open_file: String,

    /// Shell command to check if this editor is available (exit code 0 = available).
    /// Example: `"which code"` or `"command -v zed"`
    #[serde(default)]
    pub detect: String,

    /// If true, the editor runs inside a terminal (e.g., nvim, emacs -nw).
    /// When set, the open command is executed inside the platform terminal wrapper.
    #[serde(default)]
    pub terminal_wrapper: bool,

    /// Optional shell command to run after workspace creation for editor-specific setup.
    /// Example: `"mkdir -p {dir}/.vscode"`
    #[serde(default)]
    pub setup_workspace: Option<String>,
}

/// User-level editor configuration file (`~/.centy/editors.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserEditorConfig {
    /// Custom editor definitions
    #[serde(default)]
    pub editors: Vec<EditorConfig>,

    /// Default editor ID to use when no project-level override is set
    #[serde(default)]
    pub default_editor: Option<String>,
}

/// Read the user-level editor config from `~/.centy/editors.json`.
pub async fn read_user_editor_config() -> Option<UserEditorConfig> {
    let home = dirs::home_dir()?;
    let config_path = home.join(".centy").join("editors.json");

    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&config_path).await.ok()?;
    serde_json::from_str(&content).ok()
}

/// Get all available editors by merging built-in defaults with user-defined editors.
/// User editors with matching IDs override built-in defaults.
pub async fn get_all_editors() -> Vec<EditorConfig> {
    let builtins = super::builtin_editors::builtin_editors();
    let user_config = read_user_editor_config().await;

    let mut editors = builtins;

    if let Some(config) = user_config {
        for user_editor in config.editors {
            // Override built-in if same ID, otherwise append
            if let Some(pos) = editors.iter().position(|e| e.id == user_editor.id) {
                editors[pos] = user_editor;
            } else {
                editors.push(user_editor);
            }
        }
    }

    editors
}

/// Find an editor config by ID from the merged editor list.
pub async fn find_editor(editor_id: &str) -> Option<EditorConfig> {
    get_all_editors()
        .await
        .into_iter()
        .find(|e| e.id == editor_id)
}

/// Resolve the effective editor ID given an explicit choice, project default, and user default.
///
/// Priority: explicit editor_id > project default_editor > user default_editor > "vscode"
pub async fn resolve_editor_id(
    explicit_editor_id: Option<&str>,
    project_default: Option<&str>,
) -> String {
    if let Some(id) = explicit_editor_id {
        if !id.is_empty() {
            return id.to_string();
        }
    }

    if let Some(id) = project_default {
        if !id.is_empty() {
            return id.to_string();
        }
    }

    if let Some(config) = read_user_editor_config().await {
        if let Some(id) = config.default_editor {
            if !id.is_empty() {
                return id;
            }
        }
    }

    "vscode".to_string()
}

/// Check if a specific editor is available on the current system.
pub fn is_editor_available(editor: &EditorConfig) -> bool {
    // Special handling for built-in editors with richer detection
    match editor.id.as_str() {
        "vscode" => return find_vscode_binary().is_some(),
        "terminal" => return super::terminal::is_terminal_available(),
        _ => {}
    }

    if editor.detect.is_empty() {
        return false;
    }

    // Run the detect command and check exit code
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").arg("/C").arg(&editor.detect).output();
    #[cfg(not(target_os = "windows"))]
    let result = Command::new("sh").arg("-c").arg(&editor.detect).output();

    result.map(|o| o.status.success()).unwrap_or(false)
}

/// Open a workspace directory in the specified editor.
///
/// Returns `true` if the editor was successfully launched, `false` otherwise.
pub fn open_editor_by_config(editor: &EditorConfig, workspace_path: &Path) -> bool {
    let dir_str = workspace_path.to_string_lossy();

    // Special handling for built-in "vscode" to preserve the sophisticated binary detection
    if editor.id == "vscode" {
        return super::vscode::open_vscode(workspace_path).unwrap_or(false);
    }

    // Special handling for built-in "terminal"
    if editor.id == "terminal" {
        return super::terminal::open_terminal(workspace_path).unwrap_or(false);
    }

    // "none" editor - don't open anything
    if editor.id == "none" {
        return false;
    }

    let escaped_dir = super::terminal::escape_path_for_shell(&dir_str);
    let command_str = editor.open_dir.replace("{dir}", &escaped_dir);

    if editor.terminal_wrapper {
        // Run inside a platform terminal
        return open_platform_terminal(workspace_path, &command_str).unwrap_or(false);
    }

    // Run the command directly
    launch_shell_command(&command_str)
}

/// Run editor-specific workspace setup if configured.
pub async fn run_editor_setup(editor: &EditorConfig, workspace_path: &Path) {
    // Special handling for built-in vscode
    if editor.id == "vscode" {
        if let Err(e) = setup_vscode_config(workspace_path).await {
            warn!("Failed to setup VS Code config: {e}");
        }
        return;
    }

    if let Some(ref setup_cmd) = editor.setup_workspace {
        let dir_str = workspace_path.to_string_lossy();
        let escaped_dir = super::terminal::escape_path_for_shell(&dir_str);
        let cmd = setup_cmd.replace("{dir}", &escaped_dir);

        #[cfg(target_os = "windows")]
        let result = Command::new("cmd").arg("/C").arg(&cmd).output();
        #[cfg(not(target_os = "windows"))]
        let result = Command::new("sh").arg("-c").arg(&cmd).output();

        if let Err(e) = result {
            warn!("Failed to run editor setup command: {e}");
        }
    }
}

/// Launch a shell command (used for custom editor open commands).
fn launch_shell_command(command: &str) -> bool {
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").arg("/C").arg(command).spawn();
    #[cfg(not(target_os = "windows"))]
    let result = Command::new("sh").arg("-c").arg(command).spawn();

    result.is_ok()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_config_serialization() {
        let config = EditorConfig {
            id: "zed".to_string(),
            name: "Zed".to_string(),
            description: "Zed editor".to_string(),
            open_dir: "zed {dir}".to_string(),
            open_file: "zed {file}:{line}".to_string(),
            detect: "which zed".to_string(),
            terminal_wrapper: false,
            setup_workspace: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"id\":\"zed\""));
        assert!(json.contains("openDir"));

        let deserialized: EditorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "zed");
        assert_eq!(deserialized.open_dir, "zed {dir}");
    }

    #[test]
    fn test_user_editor_config_serialization() {
        let config = UserEditorConfig {
            editors: vec![EditorConfig {
                id: "neovim".to_string(),
                name: "Neovim".to_string(),
                description: "Neovim in terminal".to_string(),
                open_dir: "nvim {dir}".to_string(),
                open_file: "nvim +{line} {file}".to_string(),
                detect: "which nvim".to_string(),
                terminal_wrapper: true,
                setup_workspace: None,
            }],
            default_editor: Some("neovim".to_string()),
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: UserEditorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.editors.len(), 1);
        assert_eq!(deserialized.default_editor, Some("neovim".to_string()));
    }

    #[test]
    fn test_user_editor_config_defaults() {
        let json = "{}";
        let config: UserEditorConfig = serde_json::from_str(json).unwrap();
        assert!(config.editors.is_empty());
        assert!(config.default_editor.is_none());
    }

    #[test]
    fn test_editor_config_minimal() {
        // Only required fields
        let json = r#"{"id":"test","name":"Test","openDir":"test {dir}"}"#;
        let config: EditorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.id, "test");
        assert!(!config.terminal_wrapper);
        assert!(config.setup_workspace.is_none());
        assert!(config.detect.is_empty());
    }
}
