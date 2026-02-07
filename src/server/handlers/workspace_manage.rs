use crate::server::proto::{
    CloseTempWorkspaceRequest, CloseTempWorkspaceResponse, EditorInfo,
    EditorType as ProtoEditorType, GetSupportedEditorsRequest, GetSupportedEditorsResponse,
    ListTempWorkspacesRequest, ListTempWorkspacesResponse, TempWorkspace as ProtoTempWorkspace,
};
use crate::workspace::{
    cleanup_workspace as internal_cleanup_workspace, get_all_editors, is_editor_available,
    list_workspaces as internal_list_workspaces,
};
use tonic::{Response, Status};

pub async fn get_supported_editors(
    _req: GetSupportedEditorsRequest,
) -> Result<Response<GetSupportedEditorsResponse>, Status> {
    let all_editors = get_all_editors().await;
    let editors: Vec<EditorInfo> = all_editors
        .iter()
        .map(|e| {
            let editor_type = match e.id.as_str() {
                "vscode" => ProtoEditorType::Vscode,
                "terminal" => ProtoEditorType::Terminal,
                _ => ProtoEditorType::Unspecified,
            };
            EditorInfo {
                editor_type: editor_type.into(),
                name: e.name.clone(),
                description: e.description.clone(),
                available: is_editor_available(e),
                editor_id: e.id.clone(),
                terminal_wrapper: e.terminal_wrapper,
            }
        })
        .collect();

    Ok(Response::new(GetSupportedEditorsResponse { editors }))
}
pub async fn list_temp_workspaces(
    req: ListTempWorkspacesRequest,
) -> Result<Response<ListTempWorkspacesResponse>, Status> {
    let source_filter = if req.source_project_path.is_empty() {
        None
    } else {
        Some(req.source_project_path.as_str())
    };

    match internal_list_workspaces(req.include_expired, source_filter).await {
        Ok(workspaces) => {
            let expired_count = workspaces.iter().filter(|(_, _, exp)| *exp).count() as u32;
            let proto_workspaces: Vec<ProtoTempWorkspace> = workspaces
                .into_iter()
                .map(|(path, entry, _)| ProtoTempWorkspace {
                    workspace_path: path,
                    source_project_path: entry.source_project_path,
                    issue_id: entry.issue_id,
                    issue_display_number: entry.issue_display_number,
                    issue_title: entry.issue_title,
                    agent_name: entry.agent_name,
                    action: match entry.action.as_str() {
                        "plan" => 1,       // LLM_ACTION_PLAN
                        "implement" => 2,  // LLM_ACTION_IMPLEMENT
                        "standalone" => 0, // No specific action for standalone
                        _ => 0,
                    },
                    created_at: entry.created_at,
                    expires_at: entry.expires_at,
                    is_standalone: entry.is_standalone,
                    workspace_id: entry.workspace_id,
                    workspace_name: entry.workspace_name,
                    workspace_description: entry.workspace_description,
                })
                .collect();

            Ok(Response::new(ListTempWorkspacesResponse {
                total_count: proto_workspaces.len() as u32,
                workspaces: proto_workspaces,
                expired_count,
            }))
        }
        Err(e) => Err(Status::internal(e.to_string())),
    }
}

pub async fn close_temp_workspace(
    req: CloseTempWorkspaceRequest,
) -> Result<Response<CloseTempWorkspaceResponse>, Status> {
    match internal_cleanup_workspace(&req.workspace_path, req.force).await {
        Ok(result) => Ok(Response::new(CloseTempWorkspaceResponse {
            success: result.error.is_none(),
            error: result.error.unwrap_or_default(),
            worktree_removed: result.worktree_removed,
            directory_removed: result.directory_removed,
        })),
        Err(e) => Ok(Response::new(CloseTempWorkspaceResponse {
            success: false,
            error: e.to_string(),
            worktree_removed: false,
            directory_removed: false,
        })),
    }
}
