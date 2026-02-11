use std::path::Path;

use crate::config::read_config;
use crate::item::entities::issue::{update_issue, UpdateIssueOptions};
use crate::registry::track_project_async;
use crate::server::proto::{OpenAgentInTerminalRequest, OpenAgentInTerminalResponse};
use crate::server::resolve::resolve_issue;
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn open_agent_in_terminal(
    req: OpenAgentInTerminalRequest,
) -> Result<Response<OpenAgentInTerminalResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    let issue = match resolve_issue(project_path, &req.issue_id).await {
        Ok(i) => i,
        Err(e) => {
            return Ok(agent_err_response(
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
        .is_none_or(|c| c.workspace.update_status_on_open.is_none());
    if requires_status_config {
        return Ok(agent_err_response(
            String::new(),
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

    let agent_name = if req.agent_name.is_empty() {
        "claude".to_string()
    } else {
        req.agent_name.clone()
    };

    super::workspace_agent_open::open_workspace_and_terminal(
        project_path,
        &req,
        &issue,
        &agent_name,
    )
    .await
}

pub fn agent_err_response(
    error: String,
    issue_id: String,
    dn: u32,
    req_cfg: bool,
) -> Response<OpenAgentInTerminalResponse> {
    Response::new(OpenAgentInTerminalResponse {
        success: false,
        error,
        working_directory: String::new(),
        issue_id,
        display_number: dn,
        agent_command: String::new(),
        terminal_opened: false,
        expires_at: String::new(),
        requires_status_config: req_cfg,
    })
}
