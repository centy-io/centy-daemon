//! Workspace management via worktree-io.
pub mod data;

use crate::item::entities::issue::Issue;
use std::path::PathBuf;
use thiserror::Error;
use worktree_io::{IssueRef, Workspace};

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Git error: {0}")]
    GitError(String),
    #[error("Issue error: {0}")]
    IssueError(#[from] crate::item::entities::issue::IssueCrudError),
}

pub struct CreateWorkspaceOptions {
    pub source_project_path: PathBuf,
    pub issue: Issue,
}

pub struct CreateWorkspaceResult {
    pub workspace_path: PathBuf,
    pub workspace_reused: bool,
}

/// Create (or reopen) a git worktree for the given issue, then copy `.centy/` data into it.
pub async fn create_temp_workspace(options: CreateWorkspaceOptions) -> Result<CreateWorkspaceResult, WorkspaceError> {
    let issue_ref = IssueRef::Local {
        project_path: options.source_project_path.clone(),
        display_number: options.issue.metadata.display_number,
    };
    let workspace = Workspace::open_or_create(issue_ref).map_err(|e| WorkspaceError::GitError(e.to_string()))?;
    data::copy_issue_data_to_workspace(&options.source_project_path, &workspace.path, &options.issue.id).await?;
    Ok(CreateWorkspaceResult { workspace_path: workspace.path, workspace_reused: !workspace.created })
}

pub struct CreateStandaloneWorkspaceOptions {
    pub source_project_path: PathBuf,
    pub name: Option<String>,
}

pub struct CreateStandaloneWorkspaceResult {
    pub workspace_path: PathBuf,
    pub workspace_id: String,
    pub workspace_name: String,
    pub workspace_reused: bool,
}

/// Create (or reopen) a standalone git worktree (not tied to an issue).
pub async fn create_standalone_workspace(options: CreateStandaloneWorkspaceOptions) -> Result<CreateStandaloneWorkspaceResult, WorkspaceError> {
    let workspace_id = uuid::Uuid::new_v4().to_string();
    let workspace_name = options.name.clone().unwrap_or_else(|| format!("standalone-{workspace_id}"));
    let project_path = &options.source_project_path;
    let project_name = project_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "project".to_string());
    let workspace_path = dirs::home_dir()
        .ok_or(WorkspaceError::GitError("Could not determine home directory".to_string()))?
        .join("worktrees").join("local").join(&project_name).join(&workspace_name);
    if workspace_path.exists() {
        data::copy_project_config_to_workspace(project_path, &workspace_path).await?;
        return Ok(CreateStandaloneWorkspaceResult { workspace_path, workspace_id, workspace_name, workspace_reused: true });
    }
    if let Some(parent) = workspace_path.parent() { std::fs::create_dir_all(parent)?; }
    let branch = format!("standalone-{workspace_id}");
    let branch_exists = worktree_io::git::branch_exists_local(project_path, &branch);
    worktree_io::git::create_local_worktree(project_path, &workspace_path, &branch, branch_exists)
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;
    data::copy_project_config_to_workspace(project_path, &workspace_path).await?;
    Ok(CreateStandaloneWorkspaceResult { workspace_path, workspace_id, workspace_name, workspace_reused: false })
}

/// Remove a git worktree by path. Returns `(worktree_removed, directory_removed)`.
pub async fn remove_workspace(path: &str, force: bool) -> (bool, bool) {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["-C", path, "worktree", "remove"]);
    if force { cmd.arg("--force"); }
    cmd.arg(path);
    let worktree_removed = cmd.status().map(|s| s.success()).unwrap_or(false);
    let path_buf = std::path::Path::new(path);
    let directory_removed = if path_buf.exists() {
        tokio::fs::remove_dir_all(path_buf).await.is_ok()
    } else { true };
    (worktree_removed, directory_removed)
}
