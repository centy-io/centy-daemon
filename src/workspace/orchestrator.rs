//! Workspace creation orchestration using gwq.
//!
//! This module provides the main entry points for creating workspaces:
//! - `create_temp_workspace`: Create a workspace for working on an issue
//! - `create_standalone_workspace`: Create a workspace not tied to an issue
//!
//! It orchestrates the various components:
//! - gwq client for git worktree management
//! - Metadata layer for centy-specific data (TTL, issue binding, etc.)
//! - Path generation (from `path` module)
//! - Data copying (from `data` module)
//! - Editor launching (from `editor` module)
//! - VS Code configuration setup

use super::data::{copy_issue_data_to_workspace, copy_project_config_to_workspace};
use super::editor::{open_editor, run_editor_setup_by_id, EditorType};
use super::gwq_client::{GwqClient, GwqError};
use super::metadata::{
    find_metadata_for_issue, find_standalone_metadata, get_default_ttl_from_metadata,
    save_metadata, update_metadata_expiration, WorkspaceMetadata,
};
use super::path::{
    calculate_expires_at, generate_standalone_workspace_path, generate_workspace_path,
};
use super::types::{TempWorkspaceEntry, DEFAULT_TTL_HOURS};
use super::WorkspaceError;
use crate::common::git::is_git_repository;
use crate::item::entities::issue::Issue;
use crate::utils::now_iso;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

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

/// Get the gwq client, falling back to direct git commands if gwq is not available.
fn get_gwq_client() -> Result<GwqClient, GwqError> {
    GwqClient::new()
}

/// Set up a new issue-based workspace with data, config, and metadata.
async fn setup_new_workspace(
    gwq: &GwqClient,
    source_path: &Path,
    workspace_path: &Path,
    options: &CreateWorkspaceOptions,
    ttl_hours: u32,
) -> Result<TempWorkspaceEntry, WorkspaceError> {
    let issue_id = &options.issue.id;
    let display_number = options.issue.metadata.display_number;

    // Copy issue data to workspace
    copy_issue_data_to_workspace(source_path, workspace_path, issue_id).await?;

    // Run editor-specific workspace setup
    run_editor_setup_by_id(options.editor.to_id(), workspace_path).await;

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

    // Save metadata
    let workspace_path_str = workspace_path.to_string_lossy().to_string();
    let metadata = WorkspaceMetadata::from_entry(&workspace_path_str, &entry);
    save_metadata(&workspace_path_str, metadata).await?;

    // Also add to legacy storage for backwards compatibility
    super::storage::add_workspace(&workspace_path_str, entry.clone()).await?;

    info!(
        "Created workspace for issue #{} at {}",
        display_number,
        workspace_path.display()
    );

    // Prune stale worktrees after successful creation
    if let Err(e) = gwq.prune(source_path) {
        warn!("Failed to prune worktrees: {e}");
    }

    Ok(entry)
}

/// Set up a standalone workspace with project config and metadata.
async fn setup_standalone_workspace(
    gwq: &GwqClient,
    source_path: &Path,
    workspace_path: &Path,
    options: &CreateStandaloneWorkspaceOptions,
    ttl_hours: u32,
    workspace_id: &str,
) -> Result<TempWorkspaceEntry, WorkspaceError> {
    // Copy project config (no issue data)
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

    // Save metadata
    let workspace_path_str = workspace_path.to_string_lossy().to_string();
    let metadata = WorkspaceMetadata::from_entry(&workspace_path_str, &entry);
    save_metadata(&workspace_path_str, metadata).await?;

    // Also add to legacy storage for backwards compatibility
    super::storage::add_workspace(&workspace_path_str, entry.clone()).await?;

    info!(
        "Created standalone workspace '{}' at {}",
        entry.workspace_name,
        workspace_path.display()
    );

    // Prune stale worktrees after successful creation
    if let Err(e) = gwq.prune(source_path) {
        warn!("Failed to prune worktrees: {e}");
    }

    Ok(entry)
}

/// Handle worktree creation via gwq, including conflict resolution.
fn handle_worktree_creation(
    gwq: &GwqClient,
    source_path: &Path,
    workspace_path: &Path,
) -> Result<bool, WorkspaceError> {
    match gwq.add_worktree_at_path(source_path, workspace_path, "HEAD") {
        Ok(()) => Ok(false), // Not reused
        Err(e) => {
            let error_msg = e.to_string();
            let is_conflict = error_msg.contains("already exists")
                || error_msg.contains("already registered worktree");

            if is_conflict && workspace_path.exists() {
                info!("Reusing existing worktree at {}", workspace_path.display());
                return Ok(true); // Reusing existing worktree
            }

            // Try pruning and retrying
            if is_conflict {
                info!("Worktree conflict detected, pruning and retrying");
                if gwq.prune(source_path).is_ok() {
                    return gwq
                        .add_worktree_at_path(source_path, workspace_path, "HEAD")
                        .map(|()| false)
                        .map_err(|e| WorkspaceError::GitError(e.to_string()));
                }
            }

            Err(WorkspaceError::GitError(error_msg))
        }
    }
}

/// Create a temporary workspace for working on an issue.
///
/// This function:
/// 1. Checks if an existing workspace for this issue already exists (reopen if so)
/// 2. Validates the source project is a git repository
/// 3. Creates a git worktree at a temporary location via gwq
/// 4. Sets up editor configuration (VS Code tasks.json for VS Code mode)
/// 5. Records workspace metadata
/// 6. Opens the selected editor (VS Code, Terminal, or None)
pub async fn create_temp_workspace(
    options: CreateWorkspaceOptions,
) -> Result<CreateWorkspaceResult, WorkspaceError> {
    let source_path = &options.source_project_path;

    // Validate source project
    if !is_git_repository(source_path) {
        return Err(WorkspaceError::NotGitRepository);
    }
    if !source_path.exists() {
        return Err(WorkspaceError::SourceProjectNotFound(
            source_path.to_string_lossy().to_string(),
        ));
    }

    // Get gwq client
    let gwq = get_gwq_client().map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    // Determine TTL
    let ttl_hours = if options.ttl_hours == 0 {
        get_default_ttl_from_metadata()
            .await
            .unwrap_or(DEFAULT_TTL_HOURS)
    } else {
        options.ttl_hours
    };

    let issue_id = &options.issue.id;
    let source_path_str = source_path.to_string_lossy().to_string();

    // Check for existing workspace for this issue (in metadata)
    if let Ok(Some((existing_path, existing_metadata))) =
        find_metadata_for_issue(&source_path_str, issue_id).await
    {
        let workspace_path = PathBuf::from(&existing_path);

        // Verify the worktree still exists
        if workspace_path.exists() && gwq.is_worktree(&workspace_path) {
            let original_created_at = existing_metadata.created_at.clone();
            let new_expires_at = calculate_expires_at(ttl_hours);

            // Update expiration in both metadata and legacy storage
            update_metadata_expiration(&existing_path, &new_expires_at).await?;
            super::storage::update_workspace_expiration(&existing_path, &new_expires_at).await?;

            info!(
                "Reopening existing workspace for issue #{} at {}",
                options.issue.metadata.display_number,
                workspace_path.display()
            );

            let entry = TempWorkspaceEntry {
                expires_at: new_expires_at,
                ..existing_metadata.to_entry()
            };

            return Ok(CreateWorkspaceResult {
                workspace_path: workspace_path.clone(),
                entry,
                editor_opened: open_editor(options.editor, &workspace_path),
                workspace_reused: true,
                original_created_at: Some(original_created_at),
            });
        }
    }

    // Generate workspace path
    let workspace_path =
        generate_workspace_path(source_path, options.issue.metadata.display_number);

    // Create worktree via gwq
    let worktree_reused = handle_worktree_creation(&gwq, source_path, &workspace_path)?;

    // Setup workspace
    let entry =
        setup_new_workspace(&gwq, source_path, &workspace_path, &options, ttl_hours).await?;

    // Open editor
    let editor_opened = open_editor(options.editor, &workspace_path);

    Ok(CreateWorkspaceResult {
        workspace_path,
        entry,
        editor_opened,
        workspace_reused: worktree_reused,
        original_created_at: None,
    })
}

/// Create a standalone workspace (not tied to an issue).
///
/// This function:
/// 1. Checks if an existing standalone workspace with the same name exists (reopen if so)
/// 2. Validates the source project is a git repository
/// 3. Creates a git worktree at a temporary location via gwq
/// 4. Copies project config (shared assets, config.json)
/// 5. Records workspace metadata
/// 6. Opens the selected editor (VS Code, Terminal, or None)
pub async fn create_standalone_workspace(
    options: CreateStandaloneWorkspaceOptions,
) -> Result<CreateStandaloneWorkspaceResult, WorkspaceError> {
    let source_path = &options.source_project_path;

    // Validate source project
    if !is_git_repository(source_path) {
        return Err(WorkspaceError::NotGitRepository);
    }
    if !source_path.exists() {
        return Err(WorkspaceError::SourceProjectNotFound(
            source_path.to_string_lossy().to_string(),
        ));
    }

    // Get gwq client
    let gwq = get_gwq_client().map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    // Determine TTL
    let ttl_hours = if options.ttl_hours == 0 {
        get_default_ttl_from_metadata()
            .await
            .unwrap_or(DEFAULT_TTL_HOURS)
    } else {
        options.ttl_hours
    };

    let source_path_str = source_path.to_string_lossy().to_string();

    // Check for existing standalone workspace with the same name
    if let Some(ref name) = options.name {
        if let Ok(Some((existing_path, existing_metadata))) =
            find_standalone_metadata(&source_path_str, None, Some(name)).await
        {
            let workspace_path = PathBuf::from(&existing_path);

            // Verify the worktree still exists
            if workspace_path.exists() && gwq.is_worktree(&workspace_path) {
                let original_created_at = existing_metadata.created_at.clone();
                let new_expires_at = calculate_expires_at(ttl_hours);

                // Update expiration in both metadata and legacy storage
                update_metadata_expiration(&existing_path, &new_expires_at).await?;
                super::storage::update_workspace_expiration(&existing_path, &new_expires_at)
                    .await?;

                info!(
                    "Reopening existing standalone workspace '{}' at {}",
                    name,
                    workspace_path.display()
                );

                let entry = TempWorkspaceEntry {
                    expires_at: new_expires_at,
                    ..existing_metadata.to_entry()
                };

                return Ok(CreateStandaloneWorkspaceResult {
                    workspace_path: workspace_path.clone(),
                    entry,
                    editor_opened: open_editor(options.editor, &workspace_path),
                    workspace_reused: true,
                    original_created_at: Some(original_created_at),
                });
            }
        }
    }

    // Generate a new workspace ID
    let workspace_id = Uuid::new_v4().to_string();

    // Generate workspace path
    let workspace_path = generate_standalone_workspace_path(source_path, options.name.as_deref());

    // Create worktree via gwq
    let worktree_reused = handle_worktree_creation(&gwq, source_path, &workspace_path)?;

    // Setup workspace
    let entry = setup_standalone_workspace(
        &gwq,
        source_path,
        &workspace_path,
        &options,
        ttl_hours,
        &workspace_id,
    )
    .await?;

    // Open editor
    let editor_opened = open_editor(options.editor, &workspace_path);

    Ok(CreateStandaloneWorkspaceResult {
        workspace_path,
        entry,
        editor_opened,
        workspace_reused: worktree_reused,
        original_created_at: None,
    })
}
