//! Data copying for workspace setup.
//!
//! Provides functionality to copy issue-related data and project configuration
//! from the source project to the workspace.

use super::WorkspaceError;
use crate::item::entities::issue::copy_assets_folder;
use std::path::Path;
use tokio::fs;

/// Copy issue data from source project to workspace.
///
/// Copies:
/// - Issue file: `.centy/issues/{issue_id}.md`
/// - Shared assets: `.centy/assets/`
/// - Project config: `.centy/config.json`
pub async fn copy_issue_data_to_workspace(
    source_project: &Path,
    workspace_path: &Path,
    issue_id: &str,
) -> Result<(), WorkspaceError> {
    let source_centy = source_project.join(".centy");
    let target_centy = workspace_path.join(".centy");

    // Create target .centy directory
    fs::create_dir_all(&target_centy).await?;

    // Copy issue file if it exists
    let source_issue_file = source_centy.join("issues").join(format!("{issue_id}.md"));
    if source_issue_file.exists() {
        let target_issues_dir = target_centy.join("issues");
        fs::create_dir_all(&target_issues_dir).await?;
        fs::copy(&source_issue_file, target_issues_dir.join(format!("{issue_id}.md"))).await?;
    }

    // Copy shared assets folder
    let source_shared_assets = source_centy.join("assets");
    let target_shared_assets = target_centy.join("assets");
    drop(copy_assets_folder(&source_shared_assets, &target_shared_assets).await);

    // Copy config.json if it exists
    let source_config = source_centy.join("config.json");
    if source_config.exists() {
        fs::copy(&source_config, target_centy.join("config.json")).await?;
    }

    Ok(())
}

/// Copy project config to workspace (for standalone workspaces).
///
/// Only copies config files, not issue data.
pub async fn copy_project_config_to_workspace(
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
    drop(copy_assets_folder(&source_shared_assets, &target_shared_assets).await);

    // Copy config.json if it exists
    let source_config = source_centy.join("config.json");
    if source_config.exists() {
        fs::copy(&source_config, target_centy.join("config.json")).await?;
    }

    Ok(())
}
