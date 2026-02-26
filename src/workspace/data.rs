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
/// - Issue folder: `.centy/issues/{issue_id}/` (issue.md, metadata.json, assets/)
/// - Shared assets: `.centy/assets/`
/// - Project config: `.centy/config.json`
#[allow(unknown_lints, max_nesting_depth)]
pub async fn copy_issue_data_to_workspace(
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
    let _ = copy_assets_folder(&source_shared_assets, &target_shared_assets).await;

    // Copy config.json if it exists
    let source_config = source_centy.join("config.json");
    if source_config.exists() {
        fs::copy(&source_config, target_centy.join("config.json")).await?;
    }

    Ok(())
}
