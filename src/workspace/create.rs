//! Workspace creation logic.
//!
//! Handles creating temporary git worktrees with editor configuration.

use super::storage::{
    add_workspace, find_standalone_workspace, find_workspace_for_issue, get_default_ttl,
    update_workspace_expiration,
};
use super::terminal::open_terminal;
use super::types::{TempWorkspaceEntry, DEFAULT_TTL_HOURS};
use super::vscode::{open_vscode, setup_vscode_config};
use super::WorkspaceError;
use crate::item::entities::issue::{copy_assets_folder, Issue};
use crate::item::entities::pr::git::{create_worktree, is_git_repository, prune_worktrees};
use crate::utils::now_iso;
use chrono::{Duration, Utc};
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

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

/// Options for creating a temporary workspace
pub struct CreateWorkspaceOptions {
    /// Path to the source project
    pub source_project_path: PathBuf,

    /// The issue being worked on
    pub issue: Issue,

    /// Action type: "plan" or "implement"
    pub action: String,

    /// Name of the agent to use
    pub agent_name: String,

    /// TTL in hours (0 = use default)
    pub ttl_hours: u32,

    /// Editor to open the workspace in (default: VS Code)
    pub editor: EditorType,
}

/// Result of creating a temporary workspace
pub struct CreateWorkspaceResult {
    /// Path to the created workspace
    pub workspace_path: PathBuf,

    /// The workspace entry that was recorded
    pub entry: TempWorkspaceEntry,

    /// Whether the editor was successfully opened (VS Code or Terminal)
    pub editor_opened: bool,

    /// Whether an existing workspace was reused instead of creating a new one
    pub workspace_reused: bool,

    /// Original creation timestamp (only set if workspace was reused)
    pub original_created_at: Option<String>,
}

/// Options for creating a standalone workspace (not tied to an issue)
pub struct CreateStandaloneWorkspaceOptions {
    /// Path to the source project
    pub source_project_path: PathBuf,

    /// Optional custom name for the workspace
    pub name: Option<String>,

    /// Optional description/goals for this workspace
    pub description: Option<String>,

    /// TTL in hours (0 = use default)
    pub ttl_hours: u32,

    /// Name of the agent to use
    pub agent_name: String,

    /// Editor to open the workspace in (default: VS Code)
    pub editor: EditorType,
}

/// Result of creating a standalone workspace
pub struct CreateStandaloneWorkspaceResult {
    /// Path to the created workspace
    pub workspace_path: PathBuf,

    /// The workspace entry that was recorded
    pub entry: TempWorkspaceEntry,

    /// Whether the editor was successfully opened
    pub editor_opened: bool,

    /// Whether an existing workspace was reused
    pub workspace_reused: bool,

    /// Original creation timestamp (only set if workspace was reused)
    pub original_created_at: Option<String>,
}

/// Extract and sanitize project name from a path.
///
/// - Uses the last directory component
/// - Replaces non-alphanumeric chars with hyphens
/// - Converts to lowercase
/// - Truncates to max 30 chars
/// - Removes leading/trailing hyphens
fn sanitize_project_name(project_path: &Path) -> String {
    let name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    // Remove consecutive hyphens and trim
    let mut result = String::new();
    let mut prev_hyphen = true; // Start true to skip leading hyphens
    for c in sanitized.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    // Remove trailing hyphen and truncate
    let result = result.trim_end_matches('-');
    if result.len() > 30 {
        // Find a clean break point (avoid cutting mid-word)
        let truncated = &result[..30];
        truncated
            .rfind('-')
            .map(|i| &truncated[..i])
            .unwrap_or(truncated)
            .to_string()
    } else {
        result.to_string()
    }
}

/// Generate a unique workspace path in the system temp directory.
///
/// Format: `{project_name}-issue-{display_number}-{short_timestamp}`
/// Example: `my-app-issue-42-20231224`
fn generate_workspace_path(project_path: &Path, issue_display_number: u32) -> PathBuf {
    let project_name = sanitize_project_name(project_path);
    let date = Utc::now().format("%Y%m%d").to_string();

    let workspace_name = format!("{project_name}-issue-{issue_display_number}-{date}");
    std::env::temp_dir().join(workspace_name)
}

/// Sanitize a workspace name for use in file paths.
///
/// - Replaces non-alphanumeric chars with hyphens
/// - Converts to lowercase
/// - Truncates to max 20 chars
fn sanitize_workspace_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    // Remove consecutive hyphens
    let mut result = String::new();
    let mut prev_hyphen = true;
    for c in sanitized.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    let result = result.trim_matches('-');
    if result.len() > 20 {
        result[..20].trim_end_matches('-').to_string()
    } else {
        result.to_string()
    }
}

/// Generate a unique workspace path for standalone workspaces.
///
/// Format: `{project_name}-{workspace_name}-{short_uuid}`
/// Example: `my-app-experiment-abc12345`
fn generate_standalone_workspace_path(
    project_path: &Path,
    workspace_name: Option<&str>,
) -> PathBuf {
    let project_name = sanitize_project_name(project_path);
    let short_uuid = &Uuid::new_v4().to_string()[..8];

    let ws_name = workspace_name
        .map(sanitize_workspace_name)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "standalone".to_string());

    let dir_name = format!("{project_name}-{ws_name}-{short_uuid}");
    std::env::temp_dir().join(dir_name)
}

/// Calculate expiration timestamp based on TTL.
fn calculate_expires_at(ttl_hours: u32) -> String {
    let expires = Utc::now() + Duration::hours(i64::from(ttl_hours));
    expires.to_rfc3339()
}

/// Open the workspace in the specified editor.
fn open_editor(editor: EditorType, workspace_path: &Path) -> bool {
    match editor {
        EditorType::VSCode => open_vscode(workspace_path).unwrap_or(false),
        EditorType::Terminal => open_terminal(workspace_path).unwrap_or(false),
        EditorType::None => false,
    }
}

/// Copy issue data from source project to workspace.
///
/// Copies:
/// - Issue folder: `.centy/issues/{issue_id}/` (issue.md, metadata.json, assets/)
/// - Shared assets: `.centy/assets/`
/// - Project config: `.centy/config.json`
async fn copy_issue_data_to_workspace(
    source_project: &Path,
    workspace_path: &Path,
    issue_id: &str,
) -> Result<(), WorkspaceError> {
    let source_centy = source_project.join(".centy");
    let target_centy = workspace_path.join(".centy");

    // Create target .centy directory
    fs::create_dir_all(&target_centy).await?;

    // Copy issue folder if it exists
    let source_issue_dir = source_centy.join("issues").join(issue_id);
    if source_issue_dir.exists() {
        let target_issue_dir = target_centy.join("issues").join(issue_id);
        fs::create_dir_all(&target_issue_dir).await?;

        // Copy issue.md
        let source_issue_md = source_issue_dir.join("issue.md");
        if source_issue_md.exists() {
            fs::copy(&source_issue_md, target_issue_dir.join("issue.md")).await?;
        }

        // Copy metadata.json
        let source_metadata = source_issue_dir.join("metadata.json");
        if source_metadata.exists() {
            fs::copy(&source_metadata, target_issue_dir.join("metadata.json")).await?;
        }

        // Copy assets folder
        let source_assets = source_issue_dir.join("assets");
        let target_assets = target_issue_dir.join("assets");
        let _ = copy_assets_folder(&source_assets, &target_assets).await;
    }

    // Copy shared assets folder
    let source_shared_assets = source_centy.join("assets");
    let target_shared_assets = target_centy.join("assets");
    let _ = copy_assets_folder(&source_shared_assets, &target_shared_assets).await;

    // Copy config.json if it exists
    let source_config = source_centy.join("config.json");
    if source_config.exists() {
        fs::copy(&source_config, target_centy.join("config.json")).await?;
    }

    Ok(())
}

/// Create a temporary workspace for working on an issue.
///
/// Set up a workspace with issue data, editor config (if VS Code), and registry entry.
async fn setup_new_workspace(
    source_path: &Path,
    workspace_path: &Path,
    options: &CreateWorkspaceOptions,
    ttl_hours: u32,
) -> Result<TempWorkspaceEntry, WorkspaceError> {
    let issue_id = &options.issue.id;
    let display_number = options.issue.metadata.display_number;

    copy_issue_data_to_workspace(source_path, workspace_path, issue_id).await?;

    // Only setup VS Code config for VS Code editor
    if options.editor == EditorType::VSCode {
        setup_vscode_config(workspace_path, issue_id, display_number, &options.action).await?;
    }

    let entry = TempWorkspaceEntry {
        source_project_path: source_path.to_string_lossy().to_string(),
        issue_id: issue_id.clone(),
        issue_display_number: display_number,
        issue_title: options.issue.title.clone(),
        agent_name: options.agent_name.clone(),
        action: options.action.clone(),
        created_at: now_iso(),
        expires_at: calculate_expires_at(ttl_hours),
        worktree_ref: "HEAD".to_string(),
        is_standalone: false,
        workspace_id: String::new(),
        workspace_name: String::new(),
        workspace_description: String::new(),
    };

    add_workspace(workspace_path.to_string_lossy().as_ref(), entry.clone()).await?;
    Ok(entry)
}

/// Handle worktree creation, including conflict resolution.
fn handle_worktree_creation(
    source_path: &Path,
    workspace_path: &Path,
) -> Result<bool, WorkspaceError> {
    match create_worktree(source_path, workspace_path, "HEAD") {
        Ok(()) => Ok(false), // Not reused
        Err(e) => {
            let error_msg = e.to_string();
            let is_conflict = error_msg.contains("already exists")
                || error_msg.contains("already registered worktree");

            if is_conflict && workspace_path.exists() {
                return Ok(true); // Reusing existing worktree
            }

            if is_conflict && prune_worktrees(source_path).is_ok() {
                return create_worktree(source_path, workspace_path, "HEAD")
                    .map(|()| false)
                    .map_err(|e| WorkspaceError::GitError(e.to_string()));
            }

            Err(WorkspaceError::GitError(error_msg))
        }
    }
}

/// This function:
/// 1. Checks if an existing workspace for this issue already exists (reopen if so)
/// 2. Validates the source project is a git repository
/// 3. Creates a git worktree at a temporary location
/// 4. Sets up editor configuration (VS Code tasks.json for VS Code mode)
/// 5. Records the workspace in the registry
/// 6. Opens the selected editor (VS Code, Terminal, or None)
pub async fn create_temp_workspace(
    options: CreateWorkspaceOptions,
) -> Result<CreateWorkspaceResult, WorkspaceError> {
    let source_path = &options.source_project_path;

    if !is_git_repository(source_path) {
        return Err(WorkspaceError::NotGitRepository);
    }
    if !source_path.exists() {
        return Err(WorkspaceError::SourceProjectNotFound(
            source_path.to_string_lossy().to_string(),
        ));
    }

    let ttl_hours = if options.ttl_hours == 0 {
        get_default_ttl().await.unwrap_or(DEFAULT_TTL_HOURS)
    } else {
        options.ttl_hours
    };

    let issue_id = &options.issue.id;
    let source_path_str = source_path.to_string_lossy().to_string();

    // Check for existing workspace for this issue
    if let Ok(Some((existing_path, existing_entry))) =
        find_workspace_for_issue(&source_path_str, issue_id).await
    {
        let workspace_path = PathBuf::from(&existing_path);
        if workspace_path.exists() {
            let original_created_at = existing_entry.created_at.clone();
            let new_expires_at = calculate_expires_at(ttl_hours);
            update_workspace_expiration(&existing_path, &new_expires_at).await?;

            return Ok(CreateWorkspaceResult {
                workspace_path: workspace_path.clone(),
                entry: TempWorkspaceEntry {
                    expires_at: new_expires_at,
                    ..existing_entry
                },
                editor_opened: open_editor(options.editor, &workspace_path),
                workspace_reused: true,
                original_created_at: Some(original_created_at),
            });
        }
    }

    let workspace_path =
        generate_workspace_path(source_path, options.issue.metadata.display_number);
    let worktree_reused = handle_worktree_creation(source_path, &workspace_path)?;

    let entry = setup_new_workspace(source_path, &workspace_path, &options, ttl_hours).await?;
    let editor_opened = open_editor(options.editor, &workspace_path);

    Ok(CreateWorkspaceResult {
        workspace_path,
        entry,
        editor_opened,
        workspace_reused: worktree_reused,
        original_created_at: None,
    })
}

/// Copy project config to workspace (for standalone workspaces).
///
/// Only copies config files, not issue data.
async fn copy_project_config_to_workspace(
    source_project: &Path,
    workspace_path: &Path,
) -> Result<(), WorkspaceError> {
    let source_centy = source_project.join(".centy");
    let target_centy = workspace_path.join(".centy");

    // Create target .centy directory
    fs::create_dir_all(&target_centy).await?;

    // Copy shared assets folder
    let source_shared_assets = source_centy.join("assets");
    let target_shared_assets = target_centy.join("assets");
    let _ = copy_assets_folder(&source_shared_assets, &target_shared_assets).await;

    // Copy config.json if it exists
    let source_config = source_centy.join("config.json");
    if source_config.exists() {
        fs::copy(&source_config, target_centy.join("config.json")).await?;
    }

    Ok(())
}

/// Set up a standalone workspace with project config and registry entry.
async fn setup_standalone_workspace(
    source_path: &Path,
    workspace_path: &Path,
    options: &CreateStandaloneWorkspaceOptions,
    ttl_hours: u32,
    workspace_id: &str,
) -> Result<TempWorkspaceEntry, WorkspaceError> {
    copy_project_config_to_workspace(source_path, workspace_path).await?;

    let workspace_name = options
        .name
        .clone()
        .unwrap_or_else(|| "Standalone Workspace".to_string());
    let workspace_description = options.description.clone().unwrap_or_default();

    let entry = TempWorkspaceEntry {
        source_project_path: source_path.to_string_lossy().to_string(),
        issue_id: String::new(),
        issue_display_number: 0,
        issue_title: String::new(),
        agent_name: options.agent_name.clone(),
        action: "standalone".to_string(),
        created_at: now_iso(),
        expires_at: calculate_expires_at(ttl_hours),
        worktree_ref: "HEAD".to_string(),
        is_standalone: true,
        workspace_id: workspace_id.to_string(),
        workspace_name,
        workspace_description,
    };

    add_workspace(workspace_path.to_string_lossy().as_ref(), entry.clone()).await?;
    Ok(entry)
}

/// Create a standalone workspace (not tied to an issue).
///
/// This function:
/// 1. Checks if an existing standalone workspace with the same name exists (reopen if so)
/// 2. Validates the source project is a git repository
/// 3. Creates a git worktree at a temporary location
/// 4. Copies project config (shared assets, config.json)
/// 5. Records the workspace in the registry
/// 6. Opens the selected editor (VS Code, Terminal, or None)
pub async fn create_standalone_workspace(
    options: CreateStandaloneWorkspaceOptions,
) -> Result<CreateStandaloneWorkspaceResult, WorkspaceError> {
    let source_path = &options.source_project_path;

    if !is_git_repository(source_path) {
        return Err(WorkspaceError::NotGitRepository);
    }
    if !source_path.exists() {
        return Err(WorkspaceError::SourceProjectNotFound(
            source_path.to_string_lossy().to_string(),
        ));
    }

    let ttl_hours = if options.ttl_hours == 0 {
        get_default_ttl().await.unwrap_or(DEFAULT_TTL_HOURS)
    } else {
        options.ttl_hours
    };

    let source_path_str = source_path.to_string_lossy().to_string();

    // Check for existing standalone workspace with the same name
    if let Some(ref name) = options.name {
        if let Ok(Some((existing_path, existing_entry))) =
            find_standalone_workspace(&source_path_str, None, Some(name)).await
        {
            let workspace_path = PathBuf::from(&existing_path);
            if workspace_path.exists() {
                let original_created_at = existing_entry.created_at.clone();
                let new_expires_at = calculate_expires_at(ttl_hours);
                update_workspace_expiration(&existing_path, &new_expires_at).await?;

                return Ok(CreateStandaloneWorkspaceResult {
                    workspace_path: workspace_path.clone(),
                    entry: TempWorkspaceEntry {
                        expires_at: new_expires_at,
                        ..existing_entry
                    },
                    editor_opened: open_editor(options.editor, &workspace_path),
                    workspace_reused: true,
                    original_created_at: Some(original_created_at),
                });
            }
        }
    }

    // Generate a new workspace ID
    let workspace_id = Uuid::new_v4().to_string();

    let workspace_path = generate_standalone_workspace_path(source_path, options.name.as_deref());
    let worktree_reused = handle_worktree_creation(source_path, &workspace_path)?;

    let entry = setup_standalone_workspace(
        source_path,
        &workspace_path,
        &options,
        ttl_hours,
        &workspace_id,
    )
    .await?;
    let editor_opened = open_editor(options.editor, &workspace_path);

    Ok(CreateStandaloneWorkspaceResult {
        workspace_path,
        entry,
        editor_opened,
        workspace_reused: worktree_reused,
        original_created_at: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_project_name_simple() {
        let path = Path::new("/home/user/my-project");
        assert_eq!(sanitize_project_name(path), "my-project");
    }

    #[test]
    fn test_sanitize_project_name_with_spaces() {
        let path = Path::new("/home/user/My Cool Project");
        assert_eq!(sanitize_project_name(path), "my-cool-project");
    }

    #[test]
    fn test_sanitize_project_name_special_chars() {
        let path = Path::new("/home/user/project_v2.0@beta!");
        assert_eq!(sanitize_project_name(path), "project-v2-0-beta");
    }

    #[test]
    fn test_sanitize_project_name_long_name() {
        let path = Path::new("/home/user/this-is-a-very-long-project-name-that-exceeds-limit");
        let result = sanitize_project_name(path);
        assert!(result.len() <= 30);
        // Should break at hyphen boundary
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn test_sanitize_project_name_leading_special() {
        let path = Path::new("/home/user/---project---");
        assert_eq!(sanitize_project_name(path), "project");
    }

    #[test]
    fn test_generate_workspace_path() {
        let project_path = Path::new("/home/user/my-app");
        let path = generate_workspace_path(project_path, 42);
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("my-app-issue-42-"));
        // Should contain date in YYYYMMDD format
        assert!(path_str.contains(&Utc::now().format("%Y%m%d").to_string()));
    }

    #[test]
    fn test_generate_workspace_path_complex_name() {
        let project_path = Path::new("/Users/dev/My Cool App");
        let path = generate_workspace_path(project_path, 1);
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("my-cool-app-issue-1-"));
    }

    #[test]
    fn test_calculate_expires_at() {
        let expires = calculate_expires_at(12);
        // Should be a valid RFC3339 timestamp
        assert!(expires.contains('T'));
        assert!(expires.contains('+') || expires.contains('Z'));
    }
}
