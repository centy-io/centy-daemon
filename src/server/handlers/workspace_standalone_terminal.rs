use crate::server::handlers::workspace_standalone::open_standalone_workspace;
use crate::server::proto::{
    OpenStandaloneWorkspaceRequest, OpenStandaloneWorkspaceResponse,
    OpenStandaloneWorkspaceWithEditorRequest,
};
use tonic::{Response, Status};

/// Deprecated: thin wrapper delegating to unified `open_standalone_workspace` with "terminal" editor.
pub async fn open_standalone_workspace_terminal(
    req: OpenStandaloneWorkspaceRequest,
) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
    open_standalone_workspace(OpenStandaloneWorkspaceWithEditorRequest {
        project_path: req.project_path,
        name: req.name,
        description: req.description,
        ttl_hours: req.ttl_hours,
        agent_name: req.agent_name,
        editor_id: "terminal".to_string(),
    })
    .await
}
