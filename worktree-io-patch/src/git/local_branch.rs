use anyhow::{bail, Context, Result};
use std::path::Path;

/// Detect the current branch of a local (non-bare) repository.
///
/// Uses `git rev-parse --abbrev-ref HEAD` and falls back to checking `main`,
/// `master`, and `develop` in that order.
///
/// # Errors
///
/// Returns an error if any git command fails to spawn or if the default branch
/// cannot be determined.
pub fn detect_local_default_branch(repo: &Path) -> Result<String> {
    let output = super::git_cmd()
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to run `git rev-parse --abbrev-ref HEAD`")?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return Ok(branch);
        }
    }

    // Fallback: check common branch names
    for candidate in ["main", "master", "develop"] {
        let output = super::git_cmd()
            .args(["-C"])
            .arg(repo)
            .args(["rev-parse", "--verify", &format!("refs/heads/{candidate}")])
            .output()
            .context("Failed to run `git rev-parse`")?;
        if output.status.success() {
            return Ok(candidate.to_string()); // LLVM_COV_EXCL_LINE
        }
    }

    bail!("Could not detect default branch for the local repository"); // LLVM_COV_EXCL_LINE
}

/// Return `true` if `branch` exists as a local branch in `repo`.
#[must_use]
pub fn branch_exists_local(repo: &Path, branch: &str) -> bool {
    super::git_cmd()
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "--verify", &format!("refs/heads/{branch}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
