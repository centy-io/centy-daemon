use crate::item::entities::issue::{update_issue, UpdateIssueOptions};
use std::path::Path;

pub(super) async fn try_update_status_on_open(
    config: Option<&crate::config::CentyConfig>,
    project_path: &Path,
    issue_id: &str,
    current_status: &str,
) {
    if let Some(cfg) = config {
        if cfg.workspace.update_status_on_open == Some(true)
            && current_status != "in-progress"
            && current_status != "closed"
        {
            let _ = update_issue(
                project_path,
                issue_id,
                UpdateIssueOptions {
                    status: Some("in-progress".to_string()),
                    ..Default::default()
                },
            )
            .await;
        }
    }
}

pub(super) fn open_editor_with_hooks(
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
    if let Some(cfg) = &config {
        if let Some(script) = &cfg.hooks.pre_open {
            let _ = run_hook(script, &hook_ctx);
        }
    }
    if editor_id.is_empty() {
        return false;
    }
    let cmd = opener::resolve_editor_command(editor_id);
    let path = Path::new(workspace_path);
    let post_script = config.as_ref().and_then(|c| c.hooks.post_open.as_ref());
    post_script.map_or_else(
        || opener::open_in_editor(path, &cmd).is_ok(),
        |script| {
            let rendered = hook_ctx.render(script);
            if matches!(opener::open_with_hook(path, &cmd, &rendered), Ok(true)) {
                true
            } else {
                let _ = run_hook(script, &hook_ctx);
                opener::open_in_editor(path, &cmd).is_ok()
            }
        },
    )
}
