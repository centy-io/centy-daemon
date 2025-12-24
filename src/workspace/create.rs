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

/// Calculate expiration timestamp based on TTL.
fn calculate_expires_at(ttl_hours: u32) -> String {
    let expires = Utc::now() + Duration::hours(i64::from(ttl_hours));
    expires.to_rfc3339()
}

/// Create a plan.md template file for the "plan" action.
///
/// This gives the AI a clear target file to write the implementation plan to.
async fn create_plan_template(
    workspace_path: &Path,
    issue_id: &str,
    display_number: u32,
    title: &str,
) -> Result<(), WorkspaceError> {
    let plan_content = format!(
        r"# Implementation Plan for Issue #{display_number}

**Issue ID**: {issue_id}
**Title**: {title}

---

## Overview

<!-- Describe the high-level approach -->

## Tasks

<!-- Break down into specific, actionable tasks -->

1.
2.
3.

## Dependencies

<!-- Note any prerequisites or blocking factors -->

## Edge Cases

<!-- Consider potential issues and how to handle them -->

## Testing Strategy

<!-- Outline how the implementation should be tested -->

---

> **Note**: After completing this plan, save it using:
> ```bash
> centy add plan {display_number} --file .centy/issues/{issue_id}/plan.md
> ```
"
    );

    let plan_path = workspace_path
        .join(".centy/issues")
        .join(issue_id)
        .join("plan.md");
    fs::write(&plan_path, plan_content).await?;

    Ok(())
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

    // Generate workspace path with project name for better identification
    let display_number = options.issue.metadata.display_number;
    let workspace_path = generate_workspace_path(source_path, display_number);

    // Create the git worktree
    create_worktree(source_path, &workspace_path, "HEAD")
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    // Copy issue data to workspace
    let issue_id = &options.issue.id;
    copy_issue_data_to_workspace(source_path, &workspace_path, issue_id).await?;

    // Set up VS Code configuration
    setup_vscode_config(&workspace_path, issue_id, display_number, &options.action).await?;

    // Create plan.md template for plan action
    if options.action == "plan" {
        create_plan_template(&workspace_path, issue_id, display_number, &options.issue.title).await?;
    }

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
