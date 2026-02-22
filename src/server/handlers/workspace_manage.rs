use crate::server::proto::{
    CloseTempWorkspaceRequest, CloseTempWorkspaceResponse, EditorInfo, EditorType,
    GetSupportedEditorsRequest, GetSupportedEditorsResponse, ListTempWorkspacesRequest,
    ListTempWorkspacesResponse,
};
use crate::workspace::remove_workspace;
use tonic::{Response, Status};

pub async fn get_supported_editors(
    _req: GetSupportedEditorsRequest,
) -> Result<Response<GetSupportedEditorsResponse>, Status> {
    let vscode_available = which::which("code").is_ok();
    let terminal_available = terminal_available();
    Ok(Response::new(GetSupportedEditorsResponse {
        editors: vec![
            EditorInfo {
                editor_type: EditorType::Vscode as i32,
                name: "VS Code".to_string(),
                description: "Open in Visual Studio Code".to_string(),
                available: vscode_available,
                editor_id: "vscode".to_string(),
                terminal_wrapper: false,
            },
            EditorInfo {
                editor_type: EditorType::Terminal as i32,
                name: "Terminal".to_string(),
                description: "Open in Terminal".to_string(),
                available: terminal_available,
                editor_id: "terminal".to_string(),
                terminal_wrapper: true,
            },
        ],
    }))
}

fn terminal_available() -> bool {
    #[cfg(target_os = "linux")]
    return which::which("gnome-terminal").is_ok()
        || which::which("konsole").is_ok()
        || which::which("xterm").is_ok();
    #[cfg(not(target_os = "linux"))]
    return true;
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
