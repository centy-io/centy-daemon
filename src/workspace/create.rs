//! Workspace creation logic.
//!
//! Handles creating temporary git worktrees with VS Code configuration.

use super::storage::{add_workspace, get_default_ttl};
use super::types::{TempWorkspaceEntry, DEFAULT_TTL_HOURS};
use super::vscode::{open_vscode, setup_vscode_config};
use super::WorkspaceError;
use crate::issue::{copy_assets_folder, Issue};
use crate::pr::git::{create_worktree, is_git_repository};
use crate::utils::now_iso;
use chrono::{Duration, Utc};
use std::path::{Path, PathBuf};
use tokio::fs;

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
}

/// Result of creating a temporary workspace
pub struct CreateWorkspaceResult {
    /// Path to the created workspace
    pub workspace_path: PathBuf,

    /// The workspace entry that was recorded
    pub entry: TempWorkspaceEntry,

    /// Whether VS Code was successfully opened
    pub vscode_opened: bool,
}

/// Generate a unique workspace path in the system temp directory.
fn generate_workspace_path(issue_id: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let short_id = if issue_id.len() > 8 {
        &issue_id[..8]
    } else {
        issue_id
    };

    let workspace_name = format!("centy-{short_id}-{timestamp}");
    std::env::temp_dir().join(workspace_name)
}

/// Calculate expiration timestamp based on TTL.
fn calculate_expires_at(ttl_hours: u32) -> String {
    let expires = Utc::now() + Duration::hours(i64::from(ttl_hours));
    expires.to_rfc3339()
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
/// This function:
/// 1. Validates the source project is a git repository
/// 2. Creates a git worktree at a temporary location
/// 3. Sets up VS Code configuration with auto-run task
/// 4. Records the workspace in the registry
/// 5. Opens VS Code (if available)
pub async fn create_temp_workspace(
    options: CreateWorkspaceOptions,
) -> Result<CreateWorkspaceResult, WorkspaceError> {
    let source_path = &options.source_project_path;

    // Validate source is a git repository
    if !is_git_repository(source_path) {
        return Err(WorkspaceError::NotGitRepository);
    }

    // Validate source project exists
    if !source_path.exists() {
        return Err(WorkspaceError::SourceProjectNotFound(
            source_path.to_string_lossy().to_string(),
        ));
    }

    // Get effective TTL
    let ttl_hours = if options.ttl_hours == 0 {
        get_default_ttl().await.unwrap_or(DEFAULT_TTL_HOURS)
    } else {
        options.ttl_hours
    };

    // Generate workspace path
    let workspace_path = generate_workspace_path(&options.issue.id);

    // Create the git worktree
    create_worktree(source_path, &workspace_path, "HEAD")
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    // Copy issue data to workspace
    let issue_id = &options.issue.id;
    copy_issue_data_to_workspace(source_path, &workspace_path, issue_id).await?;

    // Set up VS Code configuration
    let display_number = options.issue.metadata.display_number;
    setup_vscode_config(&workspace_path, issue_id, display_number, &options.action).await?;

    // Create the registry entry
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

    // Record in registry
    let workspace_path_str = workspace_path.to_string_lossy().to_string();
    add_workspace(&workspace_path_str, entry.clone()).await?;

    // Try to open VS Code (don't fail if it's not available)
    let vscode_opened = open_vscode(&workspace_path).unwrap_or(false);

    Ok(CreateWorkspaceResult {
        workspace_path,
        entry,
        vscode_opened,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_workspace_path() {
        let path = generate_workspace_path("12345678-1234-1234-1234-123456789abc");
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("centy-"));
        assert!(path_str.contains("12345678"));
    }

    #[test]
    fn test_calculate_expires_at() {
        let expires = calculate_expires_at(12);
        // Should be a valid RFC3339 timestamp
        assert!(expires.contains('T'));
        assert!(expires.contains('+') || expires.contains('Z'));
    }

    #[test]
    fn test_generate_workspace_path_short_id() {
        let path = generate_workspace_path("abc");
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("centy-abc-"));
    }
}
