//! Path helpers for org issue storage.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathError {
    #[error("Failed to determine home directory")]
    HomeDirNotFound,
}

/// Get the ~/.centy/orgs/{slug} directory
pub fn get_org_dir(org_slug: &str) -> Result<PathBuf, PathError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| PathError::HomeDirNotFound)?;
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
