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
        Ok(result) => {
            let workspace_path = result.workspace_path.to_string_lossy().to_string();
            let editor_opened = open_editor_with_hooks(
                &req.editor_id,
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
            to_error_json(&req.project_path, &e),
            String::new(),
            0,
            false,
        )),
    }
}

fn open_editor_with_hooks(
    editor_id: &str,
    workspace_path: &str,
    display_number: u32,
    project_path: &Path,
) -> bool {
    use worktree_io::{
        config::Config,
        hooks::{run_hook, HookContext},
        opener,
    };

    let config = Config::load().ok();
    let project_name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let hook_ctx = HookContext {
        owner: project_name,
        repo: String::new(),
        issue: display_number.to_string(),
        branch: format!("issue-{display_number}"),
        worktree_path: workspace_path.to_string(),
    };

    if let Some(ref cfg) = config {
        if let Some(ref script) = cfg.hooks.pre_open {
            let _ = run_hook(script, &hook_ctx);
        }
    }

    if editor_id.is_empty() {
        return false;
    }

    let cmd = opener::resolve_editor_command(editor_id);
    let path = Path::new(workspace_path);
    let post_script = config.as_ref().and_then(|c| c.hooks.post_open.as_ref());

    match post_script {
        Some(script) => {
            let rendered = hook_ctx.render(script);
            match opener::open_with_hook(path, &cmd, &rendered) {
                Ok(true) => true,
                Ok(false) | Err(_) => {
                    let _ = run_hook(script, &hook_ctx);
                    opener::open_in_editor(path, &cmd).is_ok()
                }
            }
        }
        None => opener::open_in_editor(path, &cmd).is_ok(),
    }
}
