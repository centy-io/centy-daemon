use std::path::Path;

use crate::config::read_config;
use crate::item::entities::issue::{update_issue, UpdateIssueOptions};
use crate::registry::track_project_async;
use crate::server::proto::{OpenInTempWorkspaceResponse, OpenInTempWorkspaceWithEditorRequest};
use crate::server::resolve::resolve_issue;
use crate::workspace::{
    create_temp_workspace, resolve_editor_id, CreateWorkspaceOptions, EditorType,
};
use tonic::{Response, Status};

pub async fn open_in_temp_workspace(
    req: OpenInTempWorkspaceWithEditorRequest,
) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let err_response = |error: String, issue_id: String, dn: u32, req_cfg: bool| {
        Ok(Response::new(OpenInTempWorkspaceResponse {
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
        }))
    };
    let action_str = match req.action {
        1 => "plan",
        2 => "implement",
        _ => "plan",
    };
    let issue = match resolve_issue(project_path, &req.issue_id).await {
        Ok(i) => i,
        Err(e) => return err_response(e, String::new(), 0, false),
    };
    let config = read_config(project_path).await.ok().flatten();
    let requires_status_config = config
        .as_ref()
        .map(|c| c.llm.update_status_on_start.is_none())
        .unwrap_or(true);
    if requires_status_config {
        return err_response(
            "Status update preference not configured. Run 'centy config --update-status-on-start true' to enable automatic status updates.".to_string(),
            issue.id.clone(), issue.metadata.display_number, true,
        );
    }

    if let Some(ref cfg) = config {
        if cfg.llm.update_status_on_start == Some(true)
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

    // Resolve editor ID from request, project config, or user defaults
    let project_default = config.as_ref().and_then(|c| c.default_editor.as_deref());
    let editor_id = resolve_editor_id(Some(&req.editor_id), project_default).await;

    match create_temp_workspace(CreateWorkspaceOptions {
        source_project_path: project_path.to_path_buf(),
        issue: issue.clone(),
        action: action_str.to_string(),
        agent_name,
        ttl_hours: req.ttl_hours,
        editor: EditorType::from_id(&editor_id),
    })
    .await
    {
        Ok(result) => Ok(Response::new(OpenInTempWorkspaceResponse {
            success: true,
            error: String::new(),
            workspace_path: result.workspace_path.to_string_lossy().to_string(),
            issue_id: issue.id.clone(),
            display_number: issue.metadata.display_number,
            expires_at: result.entry.expires_at,
            editor_opened: result.editor_opened,
            requires_status_config: false,
            workspace_reused: result.workspace_reused,
            original_created_at: result.original_created_at.unwrap_or_default(),
        })),
        Err(e) => err_response(e.to_string(), String::new(), 0, false),
    }
}
