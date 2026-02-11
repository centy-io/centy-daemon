use std::path::Path;

use crate::item::entities::issue::Issue;
use crate::server::proto::{
    OpenAgentInTerminalRequest, OpenAgentInTerminalResponse, WorkspaceMode,
};
use crate::server::structured_error::to_error_json;
use crate::workspace::{
    create_temp_workspace, terminal::open_terminal_with_agent, CreateWorkspaceOptions, EditorType,
};
use tonic::{Response, Status};

pub async fn open_workspace_and_terminal(
    project_path: &Path,
    req: &OpenAgentInTerminalRequest,
    issue: &Issue,
    agent_name: &str,
) -> Result<Response<OpenAgentInTerminalResponse>, Status> {
    let agent_command = agent_name.to_string();

    let workspace_mode = match req.workspace_mode {
        x if x == WorkspaceMode::Temp as i32 => WorkspaceMode::Temp,
        x if x == WorkspaceMode::Current as i32 => WorkspaceMode::Current,
        _ => WorkspaceMode::Current,
    };

    let (working_dir, expires_at) = match workspace_mode {
        WorkspaceMode::Temp => match create_temp_workspace(CreateWorkspaceOptions {
            source_project_path: project_path.to_path_buf(),
            issue: issue.clone(),
            action: "agent".to_string(),
            agent_name: agent_name.to_string(),
            ttl_hours: req.ttl_hours,
            editor: EditorType::None,
        })
        .await
        {
            Ok(r) => (r.workspace_path, Some(r.entry.expires_at)),
            Err(e) => {
                return Ok(super::workspace_agent::agent_err_response(
                    to_error_json(&project_path.to_string_lossy(), &e),
                    String::new(),
                    0,
                    false,
                ))
            }
        },
        _ => (project_path.to_path_buf(), None),
    };

    let terminal_opened = open_terminal_with_agent(
        &working_dir,
        issue.metadata.display_number,
        &agent_command,
        &[],
        None,
    )
    .unwrap_or(false);

    Ok(Response::new(OpenAgentInTerminalResponse {
        success: true,
        error: String::new(),
        working_directory: working_dir.to_string_lossy().to_string(),
        issue_id: issue.id.clone(),
        display_number: issue.metadata.display_number,
        agent_command,
        terminal_opened,
        expires_at: expires_at.unwrap_or_default(),
        requires_status_config: false,
    }))
}
