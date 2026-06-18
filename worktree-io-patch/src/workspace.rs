use anyhow::Result;
use std::path::PathBuf;

use crate::git::{
    bare_clone, branch_exists_local, branch_exists_remote, create_local_worktree, create_worktree,
    detect_default_branch, git_fetch, git_worktree_prune,
};
use crate::issue::IssueRef;
use crate::ttl::WorkspaceRegistry;

/// An open (or newly created) git worktree for a given issue.
pub struct Workspace {
    /// Absolute path to the worktree directory.
    pub path: PathBuf,
    /// The issue this workspace was opened for.
    pub issue: IssueRef,
    /// `true` if this call created the worktree; `false` if it already existed.
    pub created: bool,
}

impl Workspace {
    /// Open an existing worktree or create a fresh one.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be cloned/fetched, the branch
    /// cannot be detected, or the worktree cannot be created.
    pub fn open_or_create(issue: IssueRef) -> Result<Self> {
        let worktree_path = issue.temp_path();
        let bare_path = issue.bare_clone_path();

        // Fast path: worktree already exists
        if worktree_path.exists() {
            return Ok(Self {
                path: worktree_path,
                issue,
                created: false,
            });
        }

        // LLVM_COV_EXCL_START
        if let IssueRef::Local { project_path, .. } = &issue {
            // No bare clone — use the local repo directly.
            eprintln!("Creating local worktree at {}…", worktree_path.display());
            let branch = issue.branch_name();
            let branch_exists = branch_exists_local(project_path, &branch);
            std::fs::create_dir_all(worktree_path.parent().unwrap_or(&worktree_path))?;
            let _ = git_worktree_prune(project_path);
            create_local_worktree(project_path, &worktree_path, &branch, branch_exists)?;
        } else {
            if bare_path.exists() {
                eprintln!("Fetching origin…");
                git_fetch(&bare_path)?;
            } else {
                eprintln!(
                    "Cloning {} (bare) into {}…",
                    issue.clone_url(),
                    bare_path.display()
                );
                bare_clone(&issue.clone_url(), &bare_path)?;
            }

            let base_branch = detect_default_branch(&bare_path)?;
            eprintln!("Default branch: {base_branch}");

            let branch = issue.branch_name();
            let branch_exists = branch_exists_remote(&bare_path, &branch);

            eprintln!(
                "Creating worktree {} at {}…",
                branch,
                worktree_path.display()
            );
            let _ = git_worktree_prune(&bare_path);
            create_worktree(
                &bare_path,
                &worktree_path,
                &branch,
                &base_branch,
                branch_exists,
            )?;
        }

        if let Ok(mut registry) = WorkspaceRegistry::load() {
            registry.register(worktree_path.clone());
            let _ = registry.save();
        }

        Ok(Self {
            path: worktree_path,
            issue,
            created: true,
        })
        // LLVM_COV_EXCL_STOP
    }
}

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod workspace_tests;
