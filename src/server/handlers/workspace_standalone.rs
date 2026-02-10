use std::path::Path;

use crate::config::read_config;
use crate::registry::track_project_async;
use crate::server::proto::{
    OpenStandaloneWorkspaceResponse, OpenStandaloneWorkspaceWithEditorRequest,
};
use crate::server::structured_error::to_error_json;
use crate::workspace::{
    create_standalone_workspace, resolve_editor_id, CreateStandaloneWorkspaceOptions, EditorType,
};
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

    // Resolve editor ID from request, project config, or user defaults
    let config = read_config(project_path).await.ok().flatten();
    let project_default = config.as_ref().and_then(|c| c.default_editor.as_deref());
    let editor_id = resolve_editor_id(Some(&req.editor_id), project_default).await;

    match create_standalone_workspace(CreateStandaloneWorkspaceOptions {
        source_project_path: project_path.to_path_buf(),
        name,
        description,
        ttl_hours: req.ttl_hours,
        agent_name,
        editor: EditorType::from_id(&editor_id),
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
        Err(e) => err_response(to_error_json(&req.project_path, &e)),
    }
}
