use crate::server::handlers::workspace_temp::open_in_temp_workspace;
use crate::server::proto::{
    OpenInTempWorkspaceRequest, OpenInTempWorkspaceResponse, OpenInTempWorkspaceWithEditorRequest,
};
use tonic::{Response, Status};

/// Deprecated: thin wrapper delegating to unified `open_in_temp_workspace` with "terminal" editor.
pub async fn open_in_temp_terminal(
    req: OpenInTempWorkspaceRequest,
) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
    open_in_temp_workspace(OpenInTempWorkspaceWithEditorRequest {
        project_path: req.project_path,
        issue_id: req.issue_id,
        action: req.action,
        agent_name: req.agent_name,
        ttl_hours: req.ttl_hours,
        editor_id: "terminal".to_string(),
    })
    .await
}
