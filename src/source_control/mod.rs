//! Source control integration for generating URLs to view code in web UI.
//!
//! This module provides utilities for:
//! - Detecting which source control platform is being used (GitHub, GitLab, etc.)
//! - Generating platform-specific URLs to view folders/files
//! - Supporting multiple platforms with a consistent API

mod detection;
mod platforms;

pub use platforms::build_folder_url;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceControlError {
    #[error("Not a git repository")]
    NotGitRepository,

    #[error("No remote origin configured")]
    NoRemoteOrigin,

    #[error("Unsupported source control platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Failed to detect current branch: {0}")]
    BranchDetectionFailed(String),

    #[error("Invalid remote URL format: {0}")]
    InvalidRemoteUrl(String),

    #[error("Git error: {0}")]
    GitError(String),
}

impl From<crate::pr::git::GitError> for SourceControlError {
    fn from(err: crate::pr::git::GitError) -> Self {
        match err {
            crate::pr::git::GitError::NotGitRepository => SourceControlError::NotGitRepository,
            crate::pr::git::GitError::RemoteNotFound(_) => SourceControlError::NoRemoteOrigin,
            crate::pr::git::GitError::CurrentBranchNotFound => {
                SourceControlError::BranchDetectionFailed("Current branch not found".to_string())
            }
            other => SourceControlError::GitError(other.to_string()),
        }
    }
}
