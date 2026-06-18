use super::error::GitError;
use git2::Repository;
use std::path::Path;

/// Get the origin remote URL from a git repository.
pub fn get_remote_origin_url(project_path: &Path) -> Result<String, GitError> {
    let repo = Repository::discover(project_path).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            GitError::NotGitRepository
        } else {
            GitError::Git2Error(e.to_string())
        }
    })?;
    let remote = repo
        .find_remote("origin")
        .map_err(|_e| GitError::RemoteNotFound("origin".to_string()))?;
    remote
        .url()
        .map(ToString::to_string)
        .ok_or_else(|| GitError::RemoteNotFound("origin".to_string()))
}
