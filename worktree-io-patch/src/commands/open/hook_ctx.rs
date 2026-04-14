use anyhow::Result;
use worktree_io::{
    config::Config,
    hooks::{run_hook, HookContext},
    issue::IssueRef,
    opener,
    repo_hooks::{combined_script, RepoConfig},
};

pub(super) fn effective_hooks(
    config: &Config,
    workspace_path: &std::path::Path,
) -> (Option<String>, Option<String>) {
    let repo = RepoConfig::load_from(workspace_path).unwrap_or_default();
    let pre = combined_script(
        config.hooks.pre_open.as_deref(),
        repo.hooks.pre_open.as_ref(),
    );
    let post = combined_script(
        config.hooks.post_open.as_deref(),
        repo.hooks.post_open.as_ref(),
    );
    (pre, post)
}

pub(super) fn build_hook_context(issue: &IssueRef, worktree_path: &std::path::Path) -> HookContext {
    let (owner, repo, issue_str) = match issue {
        IssueRef::GitHub {
            owner,
            repo,
            number,
        }
        | IssueRef::GitLab {
            owner,
            repo,
            number,
        } => (owner.clone(), repo.clone(), number.to_string()),
        IssueRef::Linear { owner, repo, id } => (owner.clone(), repo.clone(), id.clone()),
        IssueRef::AzureDevOps {
            org,
            project,
            repo,
            id,
        } => (format!("{org}/{project}"), repo.clone(), id.to_string()),
        IssueRef::Jira {
            owner,
            repo,
            issue_key,
            ..
        } => (owner.clone(), repo.clone(), issue_key.clone()),
        IssueRef::Adhoc {
            owner, repo, name, ..
        } => (owner.clone(), repo.clone(), name.clone()),
        IssueRef::Local {
            project_path,
            display_number,
        } => {
            let project_name = project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            (project_name, String::new(), display_number.to_string())
        }
    };
    HookContext {
        owner,
        repo,
        issue: issue_str,
        branch: issue.branch_name(),
        worktree_path: worktree_path.to_string_lossy().into_owned(),
    }
}

pub(super) fn launch_editor(
    workspace: &std::path::Path,
    cmd: Option<&str>,
    post_hook: Option<&str>,
    background: bool,
    ctx: &HookContext,
) -> Result<()> {
    match (cmd, post_hook) {
        (Some(c), Some(s)) => {
            let rendered = ctx.render(s);
            if !opener::open_with_hook(workspace, c, &rendered, background)? {
                eprintln!("Running post:open hook…");
                run_hook(s, ctx)?;
            }
        }
        (Some(c), None) => {
            opener::open_editor_or_terminal(workspace, c, background)?;
        }
        (None, Some(s)) => {
            eprintln!("Running post:open hook…");
            run_hook(s, ctx)?;
        }
        (None, None) => {}
    }
    Ok(())
}
