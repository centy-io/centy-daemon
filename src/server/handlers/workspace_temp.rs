use std::path::Path;

use crate::config::read_config;
use crate::item::entities::issue::{update_issue, UpdateIssueOptions};
use crate::registry::track_project_async;
use crate::server::proto::{OpenInTempWorkspaceResponse, OpenInTempWorkspaceWithEditorRequest};
use crate::server::resolve::resolve_issue;
use crate::server::structured_error::{to_error_json, StructuredError};
use crate::workspace::{create_temp_workspace, CreateWorkspaceOptions};
use tonic::{Response, Status};

fn err_response(
    error: String,
    issue_id: String,
    dn: u32,
    req_cfg: bool,
) -> Response<OpenInTempWorkspaceResponse> {
    Response::new(OpenInTempWorkspaceResponse {
        success: false,
        error,
        workspace_path: String::new(),
        issue_id,
        display_number: dn,
        expires_at: String::new(),
        editor_opened: false,
        requires_status_config: req_cfg,
        workspace_reused: false,
        original_created_at: String::new(),
    })
}

pub async fn open_in_temp_workspace(
    req: OpenInTempWorkspaceWithEditorRequest,
) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    let issue = match resolve_issue(project_path, &req.issue_id).await {
        Ok(i) => i,
        Err(e) => {
            return Ok(err_response(
                to_error_json(&req.project_path, &e),
                String::new(),
                0,
                false,
            ))
        }
    };

    let config = read_config(project_path).await.ok().flatten();

    let requires_status_config = config
        .as_ref()
        .map(|c| c.workspace.update_status_on_open.is_none())
        .unwrap_or(true);
    if requires_status_config {
        return Ok(err_response(
            StructuredError::new(
                &req.project_path,
                "STATUS_CONFIG_REQUIRED",
                "Status update preference not configured. Set workspace.updateStatusOnOpen in your project config.".to_string(),
            )
            .to_json(),
            issue.id.clone(),
            issue.metadata.display_number,
            true,
        ));
    }

    if let Some(ref cfg) = config {
        if cfg.workspace.update_status_on_open == Some(true)
            && issue.metadata.status != "in-progress"
            && issue.metadata.status != "closed"
        {
            let _ = update_issue(
                project_path,
                &issue.id,
                UpdateIssueOptions {
                    status: Some("in-progress".to_string()),
                    ..Default::default()
                },
            )
            .await;
        }
    }

    match create_temp_workspace(CreateWorkspaceOptions {
        source_project_path: project_path.to_path_buf(),
        issue: issue.clone(),
    })
    .await
    {
        Ok(result) => Ok(Response::new(OpenInTempWorkspaceResponse {
            success: true,
            error: String::new(),
            workspace_path: result.workspace_path.to_string_lossy().to_string(),
            issue_id: issue.id.clone(),
            display_number: issue.metadata.display_number,
            expires_at: String::new(),
            editor_opened: false,
            requires_status_config: false,
            workspace_reused: result.workspace_reused,
            original_created_at: String::new(),
        })),
        Err(e) => Ok(err_response(
            to_error_json(&req.project_path, &e),
            String::new(),
            0,
            false,
        )),
    }
}
