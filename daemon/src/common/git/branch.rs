use super::error::GitError;
use git2::{BranchType, Repository};
use std::path::Path;

fn open_repo(project_path: &Path) -> Result<Repository, GitError> {
    Repository::discover(project_path).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            GitError::NotGitRepository
        } else {
            GitError::Git2Error(e.to_string())
        }
    })
}

/// Detect the current git branch.
pub fn detect_current_branch(project_path: &Path) -> Result<String, GitError> {
    let repo = open_repo(project_path)?;
    let head = repo.head().map_err(|_e| GitError::CurrentBranchNotFound)?;
    if head.is_branch() {
        head.shorthand()
            .map(ToString::to_string)
            .ok_or(GitError::CurrentBranchNotFound)
    } else {
        Err(GitError::CurrentBranchNotFound)
    }
}

/// Validate that a branch exists in the repository (local or remote).
pub fn validate_branch_exists(project_path: &Path, branch: &str) -> Result<bool, GitError> {
    let repo = open_repo(project_path)?;
    if repo.find_branch(branch, BranchType::Local).is_ok() {
        return Ok(true);
    }
    let remote_ref = format!("origin/{branch}");
    let found = repo.find_branch(&remote_ref, BranchType::Remote).is_ok();
    Ok(found)
}

/// Check if the given path is inside a git repository.
#[must_use]
pub fn is_git_repository(project_path: &Path) -> bool {
    Repository::discover(project_path).is_ok()
}

/// Check if the given path is the root of a git repository (not just inside one).
#[must_use]
pub fn is_git_root(project_path: &Path) -> bool {
    let Ok(repo) = Repository::discover(project_path) else {
        return false;
    };
    let Some(workdir) = repo.workdir() else {
        return false;
    };
    let canonical_project = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());
    let canonical_workdir = workdir
        .canonicalize()
        .unwrap_or_else(|_| workdir.to_path_buf());
    canonical_project == canonical_workdir
}

/// Get the default branch name (main or master).
#[must_use]
pub fn get_default_branch(project_path: &Path) -> String {
    if validate_branch_exists(project_path, "main").unwrap_or(false) {
        return "main".to_string();
    }
    if validate_branch_exists(project_path, "master").unwrap_or(false) {
        return "master".to_string();
    }
    "main".to_string()
}
