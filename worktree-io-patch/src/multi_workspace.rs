//! Multi-repo workspace creation: one unified folder across several repos.
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    git::{
        bare_clone, branch_exists_remote, create_worktree, detect_default_branch, git_fetch,
        git_worktree_prune,
    },
    issue::IssueRef,
    name_gen,
    ttl::WorkspaceRegistry,
};

/// A spec for one repo in a multi-workspace.
pub enum MultiSpec {
    /// Repo paired with an issue — creates a branch, folder is `<repo>-<id>`.
    WithIssue(IssueRef),
    /// Bare repo slug — checked out on its default branch, folder is `<repo>`.
    BareRepo {
        /// GitHub owner (org or user).
        owner: String,
        /// Repository name.
        repo: String,
    },
}

/// Create a unified workspace under `workspaces_root`, one sub-dir per spec.
/// Registers the root with `WorkspaceRegistry` for TTL-based pruning.
/// # Errors
/// Returns an error if any directory, clone, fetch, or worktree step fails.
// LLVM_COV_EXCL_START
pub fn create_multi_workspace(specs: &[MultiSpec], workspaces_root: &Path) -> Result<PathBuf> {
    let root = workspaces_root.join(name_gen::generate_name());
    fs::create_dir_all(&root)
        .with_context(|| format!("failed to create workspace root {}", root.display()))?;
    for spec in specs {
        open_one(spec, &root)?;
    }
    if let Ok(mut registry) = WorkspaceRegistry::load() {
        registry.register(root.clone());
        let _ = registry.save();
    }
    Ok(root)
}

fn github_bare_path(owner: &str, repo: &str) -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join("worktrees")
        .join("github")
        .join(owner)
        .join(repo)
}

fn open_one(spec: &MultiSpec, root: &Path) -> Result<()> {
    match spec {
        MultiSpec::WithIssue(issue) => open_one_issue(issue, root),
        MultiSpec::BareRepo { owner, repo } => open_one_bare(owner, repo, root),
    }
}

fn open_one_issue(issue: &IssueRef, root: &Path) -> Result<()> {
    let bare_path = issue.bare_clone_path();
    if bare_path.exists() {
        eprintln!("Fetching origin for {}…", bare_path.display());
        git_fetch(&bare_path)?;
    } else {
        eprintln!("Cloning {} (bare)…", issue.clone_url());
        bare_clone(&issue.clone_url(), &bare_path)?;
    }
    let base_branch = detect_default_branch(&bare_path)?;
    let branch = issue.branch_name();
    let branch_exists = branch_exists_remote(&bare_path, &branch);
    let _ = git_worktree_prune(&bare_path);
    let dest = root.join(issue.multi_dir_name());
    create_worktree(&bare_path, &dest, &branch, &base_branch, branch_exists)
        .with_context(|| format!("failed to create worktree at {}", dest.display()))
}

fn open_one_bare(owner: &str, repo: &str, root: &Path) -> Result<()> {
    let bare_path = github_bare_path(owner, repo);
    let url = format!("https://github.com/{owner}/{repo}.git");
    if bare_path.exists() {
        eprintln!("Fetching origin for {}…", bare_path.display());
        git_fetch(&bare_path)?;
    } else {
        eprintln!("Cloning {url} (bare)…");
        bare_clone(&url, &bare_path)?;
    }
    let branch = detect_default_branch(&bare_path)?;
    let _ = git_worktree_prune(&bare_path);
    let dest = root.join(repo);
    create_worktree(&bare_path, &dest, &branch, &branch, true)
        .with_context(|| format!("failed to create worktree at {}", dest.display()))
}
// LLVM_COV_EXCL_STOP
