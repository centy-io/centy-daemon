use std::path::Path;

use crate::registry::track_project_async;
use crate::server::proto::{OpenStandaloneWorkspaceRequest, OpenStandaloneWorkspaceResponse};
use crate::workspace::{create_standalone_workspace, CreateStandaloneWorkspaceOptions, EditorType};
use tonic::{Response, Status};

// Deprecated: thin wrapper delegating to unified OpenStandaloneWorkspace with "vscode" editor
pub async fn open_standalone_workspace_vscode(
    req: OpenStandaloneWorkspaceRequest,
) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    let err_response = |error: String| {
        Ok(Response::new(OpenStandaloneWorkspaceResponse {
            success: false,
            error,
            workspace_path: String::new(),
            workspace_id: String::new(),
            name: String::new(),
            expires_at: String::new(),
            editor_opened: false,
            workspace_reused: false,
            original_created_at: String::new(),
        }))
    };

    let agent_name = if req.agent_name.is_empty() {
        "claude".to_string()
    } else {
        req.agent_name.clone()
    };

    let name = if req.name.is_empty() {
        None
    } else {
        Some(req.name.clone())
    };

    let description = if req.description.is_empty() {
        None
    } else {
        Some(req.description.clone())
    };

    match create_standalone_workspace(CreateStandaloneWorkspaceOptions {
        source_project_path: project_path.to_path_buf(),
        name,
        description,
        ttl_hours: req.ttl_hours,
        agent_name,
        editor: EditorType::VSCode,
    })
    .await
    {
        Ok(result) => Ok(Response::new(OpenStandaloneWorkspaceResponse {
            success: true,
            error: String::new(),
            workspace_path: result.workspace_path.to_string_lossy().to_string(),
            workspace_id: result.entry.workspace_id.clone(),
            name: result.entry.workspace_name.clone(),
            expires_at: result.entry.expires_at,
            editor_opened: result.editor_opened,
            workspace_reused: result.workspace_reused,
            original_created_at: result.original_created_at.unwrap_or_default(),
        })),
        Err(e) => err_response(e.to_string()),
    }
}
