mod editor;
mod hook_ctx;

use anyhow::Result;
use std::time::SystemTime;
use worktree_io::{
    config::Config,
    hooks::run_hook,
    issue::{DeepLinkOptions, IssueRef},
    repo_hooks_scaffold::scaffold_if_missing,
    ttl::{self, WorkspaceRegistry},
    workspace::Workspace,
};

use hook_ctx::{build_hook_context, effective_hooks, launch_editor};

pub fn cmd_open(
    issue_ref: Option<&str>,
    force_editor: bool,
    no_hooks: bool,
    headless: bool,
) -> Result<()> {
    let (issue, deep_link_opts) = match issue_ref {
        Some(r) => IssueRef::parse_with_options(r)?,
        None => (IssueRef::from_current_repo()?, DeepLinkOptions::default()),
    };
    let workspace = Workspace::open_or_create(issue.clone())?;

    if workspace.created {
        eprintln!("Created workspace at {}", workspace.path.display());
    } else {
        eprintln!("Workspace already exists at {}", workspace.path.display());
    }

    if matches!(scaffold_if_missing(&workspace.path), Ok(true)) {
        eprintln!("created .worktree.toml (no active config — edit to enable hooks)");
    }

    let config = Config::load()?;

    if config.workspace.auto_prune {
        if let Some(ttl_val) = &config.workspace.ttl {
            if let Ok(mut registry) = WorkspaceRegistry::load() {
                let now = SystemTime::now();
                let expired = ttl::prune(&registry.workspace, ttl_val, now);
                let expired_paths: Vec<_> = expired.iter().map(|r| r.path.clone()).collect();
                for path in &expired_paths {
                    eprintln!("Pruning expired workspace at {}…", path.display());
                    let _ = std::fs::remove_dir_all(path);
                }
                registry
                    .workspace
                    .retain(|r| !expired_paths.contains(&r.path));
                let _ = registry.save();
            }
        }
    }
    let hook_ctx = build_hook_context(&issue, &workspace.path);
    let (effective_pre, effective_post) = if no_hooks || deep_link_opts.no_hooks {
        (None, None)
    } else {
        effective_hooks(&config, &workspace.path)
    };

    if let Some(script) = &effective_pre {
        eprintln!("Running pre:open hook…");
        run_hook(script, &hook_ctx)?;
    }

    if headless {
        if let Some(script) = &effective_post {
            eprintln!("Running post:open hook…");
            run_hook(script, &hook_ctx)?;
        }
        return Ok(());
    }
    let editor_cmd: Option<String> = if let Some(editor_name) = deep_link_opts.editor {
        Some(editor::resolve_editor_command(&editor_name))
    } else if force_editor || config.open.editor {
        if config.editor.command.is_none() {
            eprintln!("No editor configured. Run: worktree setup");
        }
        config.editor.command
    } else {
        None
    };
    launch_editor(
        &workspace.path,
        editor_cmd.as_deref(),
        effective_post.as_deref(),
        config.editor.background,
        &hook_ctx,
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
