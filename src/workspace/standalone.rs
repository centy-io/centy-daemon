use super::{data, types::{CreateStandaloneWorkspaceOptions, CreateStandaloneWorkspaceResult, WorkspaceError}};
/// Create (or reopen) a standalone git worktree (not tied to an issue).
pub async fn create_standalone_workspace(
    options: CreateStandaloneWorkspaceOptions,
) -> Result<CreateStandaloneWorkspaceResult, WorkspaceError> {
    let workspace_id = uuid::Uuid::new_v4().to_string();
    let workspace_name = options.name.clone().unwrap_or_else(|| format!("standalone-{workspace_id}"));
    let project_path = &options.source_project_path;
    let project_name = project_path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string());
    let workspace_path = dirs::home_dir()
        .ok_or(WorkspaceError::GitError("Could not determine home directory".to_string()))?
        .join("worktrees").join("local").join(&project_name).join(&workspace_name);
    if workspace_path.exists() {
        data::copy_project_config_to_workspace(project_path, &workspace_path).await?;
        return Ok(CreateStandaloneWorkspaceResult {
            workspace_path, workspace_id, workspace_name, workspace_reused: true,
        });
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
