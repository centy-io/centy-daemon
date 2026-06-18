use anyhow::{bail, Context, Result};
use std::path::Path;

/// Clone `url` as a bare repository into `dest`.
///
/// Also configures `remote.origin.fetch` so that `git fetch` populates
/// `refs/remotes/origin/*`, then runs an initial fetch.
///
/// # Errors
///
/// Returns an error if the destination directory cannot be created, or if any
/// of the git commands fail.
pub fn bare_clone(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let status = super::git_cmd()
        .args(["clone", "--bare", url])
        .arg(dest)
        .status()
        .context("Failed to run `git clone --bare`")?;

    if !status.success() {
        bail!("git clone --bare failed for {url}"); // LLVM_COV_EXCL_LINE
    }

    let fetch_refspec = "+refs/heads/*:refs/remotes/origin/*";
    let status = super::git_cmd()
        .args(["-C"])
        .arg(dest)
        .args(["config", "remote.origin.fetch", fetch_refspec])
        .status()
        .context("Failed to configure remote.origin.fetch")?;

    if !status.success() {
        bail!("Failed to set remote.origin.fetch"); // LLVM_COV_EXCL_LINE
    }

    let status = super::git_cmd()
        .args(["-C"])
        .arg(dest)
        .args(["fetch", "origin"])
        .status()
        .context("Failed to run `git fetch origin`")?;

    if !status.success() {
        bail!("git fetch origin failed after bare clone"); // LLVM_COV_EXCL_LINE
    }

    Ok(())
}

/// Fetch the latest refs from `origin` for a bare clone at `bare`.
///
/// # Errors
///
/// Returns an error if the git command fails to spawn or exits non-zero.
pub fn git_fetch(bare: &Path) -> Result<()> {
    let status = super::git_cmd()
        .args(["-C"])
        .arg(bare)
        .args(["fetch", "origin"])
        .status()
        .context("Failed to run `git fetch`")?;

    if !status.success() {
        bail!("git fetch origin failed"); // LLVM_COV_EXCL_LINE
    }
    Ok(())
}
