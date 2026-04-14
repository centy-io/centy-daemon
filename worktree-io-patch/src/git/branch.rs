use anyhow::{bail, Context, Result};
use std::path::Path;

/// Detect the default branch of a bare remote repository.
///
/// Tries `symbolic-ref refs/remotes/origin/HEAD`, then `git remote show origin`,
/// then falls back to checking `main`, `master`, and `develop` in that order.
///
/// # Errors
///
/// Returns an error if any git command fails to spawn or if the default branch
/// cannot be determined.
pub fn detect_default_branch(bare: &Path) -> Result<String> {
    let output = super::git_cmd()
        .args(["-C"])
        .arg(bare)
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output()
        .context("Failed to run `git symbolic-ref`")?;

    if output.status.success() {
        let full = String::from_utf8_lossy(&output.stdout);
        if let Some(branch) = full.trim().strip_prefix("refs/remotes/origin/") {
            return Ok(branch.to_string());
        }
    }

    let output = super::git_cmd()
        .args(["-C"])
        .arg(bare)
        .args(["remote", "show", "origin"])
        .output()
        .context("Failed to run `git remote show origin`")?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let line = line.trim();
            if let Some(branch) = line.strip_prefix("HEAD branch: ") {
                return Ok(branch.to_string()); // LLVM_COV_EXCL_LINE
            }
        }
    }

    for candidate in ["main", "master", "develop"] {
        let output = super::git_cmd()
            .args(["-C"])
            .arg(bare)
            .args([
                "rev-parse",
                "--verify",
                &format!("refs/remotes/origin/{candidate}"),
            ])
            .output()
            .context("Failed to run `git rev-parse`")?;
        if output.status.success() {
            return Ok(candidate.to_string()); // LLVM_COV_EXCL_LINE
        }
    }

    bail!("Could not detect default branch for the repository"); // LLVM_COV_EXCL_LINE
}

/// Return `true` if `branch` exists as a remote-tracking ref in the bare clone.
#[must_use]
pub fn branch_exists_remote(bare: &Path, branch: &str) -> bool {
    super::git_cmd()
        .args(["-C"])
        .arg(bare)
        .args([
            "rev-parse",
            "--verify",
            &format!("refs/remotes/origin/{branch}"),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
