use std::path::Path;

use crate::registry::track_project_async;
use crate::server::proto::{
    OpenStandaloneWorkspaceResponse, OpenStandaloneWorkspaceWithEditorRequest,
};
use crate::server::structured_error::to_error_json;
use crate::workspace::{create_standalone_workspace, CreateStandaloneWorkspaceOptions};
use tonic::{Response, Status};

pub async fn open_standalone_workspace(
    req: OpenStandaloneWorkspaceWithEditorRequest,
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

    let name = if req.name.is_empty() {
        None
    } else {
        Some(req.name.clone())
    };

    match create_standalone_workspace(CreateStandaloneWorkspaceOptions {
        source_project_path: project_path.to_path_buf(),
        name,
    })
    .await
    {
        Ok(result) => Ok(Response::new(OpenStandaloneWorkspaceResponse {
            success: true,
            error: String::new(),
            workspace_path: result.workspace_path.to_string_lossy().to_string(),
            workspace_id: result.workspace_id,
            name: result.workspace_name,
            expires_at: String::new(),
            editor_opened: false,
            workspace_reused: result.workspace_reused,
            original_created_at: String::new(),
        })),
        Err(e) => err_response(to_error_json(&req.project_path, &e)),
    }
}
