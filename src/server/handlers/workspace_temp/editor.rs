use std::path::Path;
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
