//! Workspace creation orchestration.
//!
//! This module provides the main entry point for creating temporary workspaces.
//! It orchestrates the various components:
//! - Path generation (from `path` module)
//! - Issue data copying (from `data` module)
//! - Editor launching (from `editor` module)
//! - Git worktree management
//! - VS Code configuration setup

use super::data::copy_issue_data_to_workspace;
use super::editor::{open_editor, EditorType};
use super::path::{calculate_expires_at, generate_workspace_path};
use super::storage::{
    add_workspace, find_workspace_for_issue, get_default_ttl, update_workspace_expiration,
};
use super::types::{TempWorkspaceEntry, DEFAULT_TTL_HOURS};
use super::vscode::setup_vscode_config;
use super::WorkspaceError;
use crate::item::entities::issue::Issue;
use crate::item::entities::pr::git::{create_worktree, is_git_repository, prune_worktrees};
use crate::utils::now_iso;
use std::path::{Path, PathBuf};

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
