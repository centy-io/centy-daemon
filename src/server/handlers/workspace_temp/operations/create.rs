use super::editor::open_editor_with_hooks;
use super::super::response::err_response;
use crate::item::entities::issue::Issue;
use crate::server::proto::OpenInTempWorkspaceResponse;
use crate::server::structured_error::to_error_json;
use crate::workspace::{create_temp_workspace, CreateWorkspaceOptions};
use std::path::Path;
use tonic::{Response, Status};

pub async fn run_create_workspace(
    project_path: &Path,
    req_project_path: &str,
    req_editor_id: &str,
    issue: Issue,
) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
    match create_temp_workspace(CreateWorkspaceOptions {
        source_project_path: project_path.to_path_buf(),
        issue: issue.clone(),
    })
    .await
    {
        Ok(result) => {
            let workspace_path = result.workspace_path.to_string_lossy().to_string();
            let editor_opened = open_editor_with_hooks(
                req_editor_id,
                &workspace_path,
                issue.metadata.display_number,
                project_path,
            );
            Ok(Response::new(OpenInTempWorkspaceResponse {
                success: true,
                error: String::new(),
                workspace_path,
                issue_id: issue.id.clone(),
                display_number: issue.metadata.display_number,
                expires_at: String::new(),
                editor_opened,
                requires_status_config: false,
                workspace_reused: result.workspace_reused,
                original_created_at: String::new(),
            }))
        }
        Err(e) => Ok(err_response(
            to_error_json(req_project_path, &e),
            String::new(),
            0,
            false,
        )),
    }
}
