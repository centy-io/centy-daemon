use crate::server::proto::{
    CloseTempWorkspaceRequest, CloseTempWorkspaceResponse, EditorInfo,
    GetSupportedEditorsRequest, GetSupportedEditorsResponse, ListTempWorkspacesRequest,
    ListTempWorkspacesResponse,
};
use crate::server::structured_error::StructuredError;
use crate::workspace::remove_workspace;
use tonic::{Response, Status};

const TERMINAL_ALIASES: &[&str] = &[
    "terminal",
    "iterm",
    "iterm2",
    "warp",
    "ghostty",
    "alacritty",
    "kitty",
    "wezterm",
    "tmux",
    "wt",
];

pub fn get_supported_editors(
    _req: GetSupportedEditorsRequest,
) -> Result<Response<GetSupportedEditorsResponse>, Status> {
    use std::collections::HashSet;
    use worktree_io::opener::{detect::available_entries, entries::all_entries};

    let available: HashSet<&str> = available_entries()
        .iter()
        .filter_map(|e| e.aliases.first().copied())
        .collect();

    let editors = all_entries()
        .into_iter()
        .filter_map(|e| {
            let primary = e.aliases.first().copied()?;
            let is_terminal = e.aliases.iter().any(|&a| TERMINAL_ALIASES.contains(&a));
            Some(EditorInfo {
                name: e.display.to_string(),
                description: format!("Open in {}", e.display),
                available: available.contains(primary),
                editor_id: primary.to_string(),
                terminal_wrapper: is_terminal,
            })
        })
        .collect();

    Ok(Response::new(GetSupportedEditorsResponse { editors }))
}

/// Returns an empty list — workspace tracking is now handled by git worktree state.
pub fn list_temp_workspaces(
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
            StructuredError::new(
                &req.workspace_path,
                "WORKSPACE_REMOVE_FAILED",
                "Failed to remove workspace".to_string(),
            )
            .to_json()
        } else {
            String::new()
        },
        worktree_removed,
        directory_removed,
    }))
}
