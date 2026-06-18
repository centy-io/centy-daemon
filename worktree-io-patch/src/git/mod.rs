mod branch;
mod clone;
mod local_branch;
mod prune;
mod remote;

pub use branch::{branch_exists_remote, detect_default_branch};
pub use clone::{bare_clone, git_fetch};
pub use local_branch::{branch_exists_local, detect_local_default_branch};
pub use prune::git_worktree_prune;
pub use remote::get_remote_url;

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Build a `git` [`Command`] with worktree-related environment variables unset.
///
/// When the process runs inside a git worktree hook, `GIT_DIR`, `GIT_WORK_TREE`,
/// and `GIT_INDEX_FILE` are set by git itself and would override the `-C <dir>`
/// flag, causing child git commands to operate on the wrong repository.
/// Clearing them here ensures `-C dir` is always honoured.
pub(super) fn git_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE");
    cmd
}

/// Create a worktree from a local (non-bare) repository without referencing a remote.
///
/// When `branch_exists` is false a new branch is created from HEAD.
/// When `branch_exists` is true the existing local branch is checked out.
///
/// # Errors
///
/// Returns an error if the git command fails to spawn or exits non-zero.
pub fn create_local_worktree(
    repo: &Path,
    dest: &Path,
    branch: &str,
    branch_exists: bool,
) -> Result<()> {
    let mut cmd = git_cmd();
    cmd.args(["-C"]).arg(repo).arg("worktree").arg("add");

    if branch_exists {
        cmd.arg(dest).arg(branch);
    } else {
        // Create new branch from HEAD (no origin/ reference needed)
        cmd.arg(dest).arg("-b").arg(branch);
    }

    let status = cmd.status().context("Failed to run `git worktree add`")?;

    if !status.success() {
        bail!("git worktree add failed for branch {branch}"); // LLVM_COV_EXCL_LINE
    }
    Ok(())
}

/// Create a worktree inside a bare clone.
///
/// When `branch_exists` is false a new branch is created from `origin/<base_branch>`.
/// When `branch_exists` is true the existing branch is checked out.
///
/// # Errors
///
/// Returns an error if the git command fails to spawn or exits non-zero.
pub fn create_worktree(
    bare: &Path,
    dest: &Path,
    branch: &str,
    base_branch: &str,
    branch_exists: bool,
) -> Result<()> {
    let mut cmd = git_cmd();
    cmd.args(["-C"]).arg(bare).arg("worktree").arg("add");

    if branch_exists {
        cmd.arg(dest).arg(branch);
    } else {
        cmd.arg(dest)
            .arg("-b")
            .arg(branch)
            .arg(format!("origin/{base_branch}"));
    }

    let status = cmd.status().context("Failed to run `git worktree add`")?;

    if !status.success() {
        bail!("git worktree add failed for branch {branch}"); // LLVM_COV_EXCL_LINE
    }
    Ok(())
}
