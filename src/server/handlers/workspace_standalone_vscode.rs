use crate::server::handlers::workspace_standalone::open_standalone_workspace;
use crate::server::proto::{
    OpenStandaloneWorkspaceRequest, OpenStandaloneWorkspaceResponse,
    OpenStandaloneWorkspaceWithEditorRequest,
};
use tonic::{Response, Status};

/// Deprecated: thin wrapper delegating to unified `open_standalone_workspace` with "vscode" editor.
pub async fn open_standalone_workspace_vscode(
    req: OpenStandaloneWorkspaceRequest,
) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
    open_standalone_workspace(OpenStandaloneWorkspaceWithEditorRequest {
        project_path: req.project_path,
        name: req.name,
        description: req.description,
        ttl_hours: req.ttl_hours,
        agent_name: req.agent_name,
        editor_id: "vscode".to_string(),
    })
    .await
}
