//! VS Code integration for temporary workspaces.
//!
//! Handles setting up VS Code with auto-running tasks and opening the editor.

use super::WorkspaceError;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// Generate the VS Code tasks.json content for auto-running an agent.
///
/// The task will automatically run when the folder is opened in VS Code.
/// User must have `"task.allowAutomaticTasks": "on"` in VS Code settings.
pub fn generate_tasks_json(issue_id: &str, display_number: u32, action: &str) -> String {
    let action_display = match action {
        "plan" => "Plan",
        "implement" => "Implement",
        _ => action,
    };

    format!(
        r#"{{
  "version": "2.0.0",
  "tasks": [
    {{
      "label": "{action_display} Issue #{display_number}",
      "type": "shell",
      "command": "centy",
      "args": ["issue", "{issue_id}", "--action", "{action}"],
      "presentation": {{
        "reveal": "always",
        "panel": "new",
        "focus": true
      }},
      "runOptions": {{
        "runOn": "folderOpen"
      }},
      "problemMatcher": []
    }}
  ]
}}"#
    )
}

/// Set up VS Code configuration in the workspace.
///
/// Creates `.vscode/tasks.json` with an auto-run task for the agent.
pub async fn setup_vscode_config(
    workspace_path: &Path,
    issue_id: &str,
    display_number: u32,
    action: &str,
) -> Result<(), WorkspaceError> {
    let vscode_dir = workspace_path.join(".vscode");
    fs::create_dir_all(&vscode_dir).await?;

    let tasks_json = generate_tasks_json(issue_id, display_number, action);
    let tasks_path = vscode_dir.join("tasks.json");
    fs::write(&tasks_path, tasks_json).await?;

    Ok(())
}

/// Find the VS Code binary by checking PATH and common installation locations.
///
/// This function searches:
/// 1. The system PATH (via `which`)
/// 2. Common installation paths for each platform
///
/// This is necessary because GUI applications (like Tauri desktop apps) don't
/// inherit the user's shell PATH modifications, so VS Code's `code` command
/// may not be found even when properly installed.
#[must_use]
pub fn find_vscode_binary() -> Option<PathBuf> {
    // First, try PATH (works when daemon is started from shell)
    if let Ok(path) = which::which("code") {
        return Some(path);
    }

    // Check common installation locations by platform
    let common_paths = get_common_vscode_paths();

    for path_str in common_paths {
        let path = Path::new(path_str);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    None
}

/// Get common VS Code installation paths for the current platform.
#[cfg(target_os = "macos")]
fn get_common_vscode_paths() -> Vec<&'static str> {
    vec![
        // Standard symlink location (created by "Install 'code' command in PATH")
        "/usr/local/bin/code",
        // Homebrew on Apple Silicon
        "/opt/homebrew/bin/code",
        // Direct app bundle path
        "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
        // User-installed location
        "~/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
        // VS Code Insiders
        "/Applications/Visual Studio Code - Insiders.app/Contents/Resources/app/bin/code-insiders",
        "/usr/local/bin/code-insiders",
    ]
}

#[cfg(target_os = "linux")]
fn get_common_vscode_paths() -> Vec<&'static str> {
    vec![
        // Standard package manager installations
        "/usr/bin/code",
        "/usr/local/bin/code",
        // Snap installation
        "/snap/bin/code",
        // Flatpak installation
        "/var/lib/flatpak/exports/bin/com.visualstudio.code",
        // Direct installation
        "/usr/share/code/bin/code",
        // VS Code Insiders
        "/usr/bin/code-insiders",
        "/snap/bin/code-insiders",
    ]
}

#[cfg(target_os = "windows")]
fn get_common_vscode_paths() -> Vec<&'static str> {
    vec![
        // User installation (most common)
        r"C:\Users\%USERNAME%\AppData\Local\Programs\Microsoft VS Code\bin\code.cmd",
        r"C:\Users\%USERNAME%\AppData\Local\Programs\Microsoft VS Code\Code.exe",
        // System installation
        r"C:\Program Files\Microsoft VS Code\bin\code.cmd",
        r"C:\Program Files\Microsoft VS Code\Code.exe",
        r"C:\Program Files (x86)\Microsoft VS Code\bin\code.cmd",
        // VS Code Insiders
        r"C:\Users\%USERNAME%\AppData\Local\Programs\Microsoft VS Code Insiders\bin\code-insiders.cmd",
    ]
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn get_common_vscode_paths() -> Vec<&'static str> {
    vec![]
}

/// Check if VS Code is available (either in PATH or common locations).
#[must_use]
pub fn is_vscode_available() -> bool {
    find_vscode_binary().is_some()
}

/// Open VS Code at the specified workspace path.
///
/// Returns Ok even if VS Code isn't installed, but sets the boolean to indicate
/// whether VS Code was actually opened.
pub fn open_vscode(workspace_path: &Path) -> Result<bool, WorkspaceError> {
    let vscode_path = match find_vscode_binary() {
        Some(path) => path,
        None => {
            // VS Code not installed, but this is not an error
            // The workspace is still created and can be opened manually
            return Ok(false);
        }
    };

    let result = Command::new(&vscode_path).arg(workspace_path).spawn();

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(WorkspaceError::VscodeError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tasks_json() {
        let json = generate_tasks_json("uuid-1234", 42, "plan");

        assert!(json.contains("Plan Issue #42"));
        assert!(json.contains(r#""command": "centy""#));
        assert!(json.contains(r#""issue", "uuid-1234""#));
        assert!(json.contains(r#""--action", "plan""#));
        assert!(json.contains(r#""runOn": "folderOpen""#));
    }

    #[test]
    fn test_generate_tasks_json_implement() {
        let json = generate_tasks_json("uuid-5678", 10, "implement");

        assert!(json.contains("Implement Issue #10"));
        assert!(json.contains(r#""--action", "implement""#));
    }

    #[test]
    fn test_tasks_json_valid_json() {
        let json = generate_tasks_json("test-uuid", 1, "plan");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["version"], "2.0.0");
        assert!(parsed["tasks"].is_array());
        assert_eq!(parsed["tasks"][0]["type"], "shell");
    }

    #[test]
    fn test_get_common_vscode_paths_not_empty() {
        // Should return non-empty list on supported platforms
        let paths = get_common_vscode_paths();
        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        assert!(!paths.is_empty());
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        assert!(paths.is_empty());
    }

    #[test]
    fn test_is_vscode_available_returns_bool() {
        // Just verify it returns without panicking
        let _ = is_vscode_available();
    }

    #[test]
    fn test_find_vscode_binary_returns_option() {
        // Just verify it returns without panicking
        let result = find_vscode_binary();
        // If found, path should exist
        if let Some(path) = result {
            assert!(path.exists());
        }
    }
}
