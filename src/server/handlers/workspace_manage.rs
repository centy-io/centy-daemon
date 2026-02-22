use crate::server::proto::{
    CloseTempWorkspaceRequest, CloseTempWorkspaceResponse, GetSupportedEditorsRequest,
    GetSupportedEditorsResponse, ListTempWorkspacesRequest, ListTempWorkspacesResponse,
};
use crate::workspace::remove_workspace;
use tonic::{Response, Status};

/// Returns an empty list — the editor system has been removed.
pub async fn get_supported_editors(
    _req: GetSupportedEditorsRequest,
) -> Result<Response<GetSupportedEditorsResponse>, Status> {
    Ok(Response::new(GetSupportedEditorsResponse { editors: vec![] }))
}

/// Returns an empty list — workspace tracking is now handled by git worktree state.
pub async fn list_temp_workspaces(
    _req: ListTempWorkspacesRequest,
) -> Result<Response<ListTempWorkspacesResponse>, Status> {
    Ok(Response::new(ListTempWorkspacesResponse {
        workspaces: vec![],
        total_count: 0,
        expired_count: 0,
        success: true,
        error: String::new(),
    }))
}

pub async fn close_temp_workspace(
    req: CloseTempWorkspaceRequest,
) -> Result<Response<CloseTempWorkspaceResponse>, Status> {
    let (worktree_removed, directory_removed) =
        remove_workspace(&req.workspace_path, req.force).await;

    Ok(Response::new(CloseTempWorkspaceResponse {
        success: worktree_removed || directory_removed,
        error: if !worktree_removed && !directory_removed {
            "Failed to remove workspace".to_string()
        } else {
            String::new()
        },
        worktree_removed,
        directory_removed,
    }))
}
