//! Handler for the `open-multi` subcommand.
use anyhow::{bail, Result};
use worktree_io::{
    config::Config,
    hooks::{run_hook, HookContext},
    issue::IssueRef,
    multi_workspace::{create_multi_workspace, MultiSpec},
    opener,
};

// LLVM_COV_EXCL_START
/// Parse one argument as an issue reference or a bare `owner/repo` slug.
/// Bare slugs are checked out on their default branch.
fn parse_spec(s: &str) -> Result<MultiSpec> {
    // Bare repo slug first: owner/repo with no issue markers.
    let has_marker = s.contains('#') || s.contains('!') || s.contains('@') || s.contains(':');
    if !has_marker {
        if let Some((owner, repo)) = s.split_once('/') {
            if !owner.is_empty() && !repo.is_empty() && !repo.contains('/') {
                return Ok(MultiSpec::BareRepo {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                });
            }
        }
    }
    if let Ok((issue, _)) = IssueRef::parse_with_options(s) {
        return Ok(MultiSpec::WithIssue(issue));
    }
    bail!("could not parse {s:?} as an issue reference or bare repo slug (owner/repo)")
}

/// Open multiple repos as a single unified workspace under
/// `~/workspaces/<random-name>/`. Each call creates a fresh workspace
/// (not idempotent by design). At least two arguments are required.
pub fn cmd_open_multi(refs: &[String], no_hooks: bool) -> Result<()> {
    if refs.len() < 2 {
        bail!("open-multi requires at least two arguments");
    }
    let specs = refs
        .iter()
        .map(|r| parse_spec(r))
        .collect::<Result<Vec<_>>>()?;

    let workspaces_root = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?
        .join("workspaces");

    let root = create_multi_workspace(&specs, &workspaces_root)?;
    eprintln!("Created unified workspace at {}", root.display());

    let config = Config::load()?;

    let hook_ctx = HookContext {
        owner: String::new(),
        repo: String::new(),
        issue: String::new(),
        branch: String::new(),
        worktree_path: root.to_string_lossy().into_owned(),
    };

    if !no_hooks {
        if let Some(script) = &config.hooks.pre_open {
            eprintln!("Running pre:open hook…");
            run_hook(script, &hook_ctx)?;
        }
    }

    let editor_cmd = if config.open.editor {
        config.editor.command
    } else {
        None
    };

    let post = if no_hooks {
        None
    } else {
        config.hooks.post_open.as_deref()
    };
    match (editor_cmd.as_deref(), post) {
        (Some(cmd), Some(script)) => {
            let rendered = hook_ctx.render(script);
            if !opener::open_with_hook(&root, cmd, &rendered, config.editor.background)? {
                eprintln!("Running post:open hook…");
                run_hook(script, &hook_ctx)?;
            }
        }
        (Some(cmd), None) => opener::open_in_editor(&root, cmd, config.editor.background)?,
        (None, Some(script)) => {
            eprintln!("Running post:open hook…");
            run_hook(script, &hook_ctx)?;
        }
        (None, None) => {}
    }

    Ok(())
}
// LLVM_COV_EXCL_STOP
