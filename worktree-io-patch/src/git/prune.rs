use anyhow::{bail, Context, Result};
use std::path::Path;

/// Prune stale worktree administrative files from `repo`.
///
/// This removes git's internal bookkeeping for any worktree whose working
/// directory no longer exists on disk, allowing a fresh `git worktree add`
/// to succeed for the same branch.
///
/// # Errors
///
/// Returns an error if the git command fails to spawn or exits non-zero.
pub fn git_worktree_prune(repo: &Path) -> Result<()> {
    let status = super::git_cmd()
        .args(["-C"])
        .arg(repo)
        .args(["worktree", "prune"])
        .status()
        .context("Failed to run `git worktree prune`")?;

    if !status.success() {
        bail!("git worktree prune failed"); // LLVM_COV_EXCL_LINE
    }
    Ok(())
}
