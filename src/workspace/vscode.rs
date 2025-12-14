//! VS Code integration for temporary workspaces.
//!
//! Handles setting up VS Code with auto-running tasks and opening the editor.

use super::WorkspaceError;
use std::path::Path;
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

/// Check if VS Code is available in PATH.
#[must_use]
pub fn is_vscode_available() -> bool {
    which::which("code").is_ok()
}

/// Open VS Code at the specified workspace path.
///
/// Returns Ok even if VS Code isn't installed, but sets the boolean to indicate
/// whether VS Code was actually opened.
pub fn open_vscode(workspace_path: &Path) -> Result<bool, WorkspaceError> {
    if !is_vscode_available() {
        // VS Code not installed, but this is not an error
        // The workspace is still created and can be opened manually
        return Ok(false);
    }

    let result = Command::new("code")
        .arg(workspace_path)
        .spawn();

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
}
