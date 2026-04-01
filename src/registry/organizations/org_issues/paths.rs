//! Path helpers for org issue storage.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathError {
    #[error("Failed to determine home directory")]
    HomeDirNotFound,
}

/// Get the ~/.centy/orgs/{slug} directory
///
/// If `CENTY_HOME` is set, that directory is used as the base instead of `~/.centy`.
/// This allows tests and CI to use an isolated directory without touching the real
/// `~/.centy` data.
pub fn get_org_dir(org_slug: &str) -> Result<PathBuf, PathError> {
    if let Ok(centy_home) = std::env::var("CENTY_HOME") {
        return Ok(PathBuf::from(centy_home).join("orgs").join(org_slug));
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_e| PathError::HomeDirNotFound)?;
    Ok(PathBuf::from(home)
        .join(".centy")
        .join("orgs")
        .join(org_slug))
}

/// Get the ~/.centy/orgs/{slug}/issues directory
pub fn get_org_issues_dir(org_slug: &str) -> Result<PathBuf, PathError> {
    Ok(get_org_dir(org_slug)?.join("issues"))
}

/// Get the ~/.centy/orgs/{slug}/config.json path
pub fn get_org_config_path(org_slug: &str) -> Result<PathBuf, PathError> {
    Ok(get_org_dir(org_slug)?.join("config.json"))
}
