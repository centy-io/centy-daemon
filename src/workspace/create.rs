use worktree_io::{IssueRef, Workspace};
use super::{data, types::{CreateWorkspaceOptions, CreateWorkspaceResult, WorkspaceError}};
/// Create (or reopen) a git worktree for the given issue.
pub async fn create_temp_workspace(
    options: CreateWorkspaceOptions,
) -> Result<CreateWorkspaceResult, WorkspaceError> {
    let issue_ref = IssueRef::Local {
        project_path: options.source_project_path.clone(),
        display_number: options.issue.metadata.display_number,
    };
    let workspace = Workspace::open_or_create(issue_ref)
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;
    data::copy_issue_data_to_workspace(
        &options.source_project_path, &workspace.path, &options.issue.id,
    ).await?;
    Ok(CreateWorkspaceResult { workspace_path: workspace.path, workspace_reused: !workspace.created })
}
