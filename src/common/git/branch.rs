use super::error::GitError;
use std::path::Path;
use std::process::Command;
/// Detect the current git branch.
/// Runs `git rev-parse --abbrev-ref HEAD` in the given project path.
pub fn detect_current_branch(project_path: &Path) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitError::NotGitRepository);
        }
        return Err(GitError::CommandError(stderr.to_string()));
    }
    let branch = String::from_utf8(output.stdout)
        .map_err(|_| GitError::InvalidUtf8)?
        .trim()
        .to_string();
    if branch.is_empty() || branch == "HEAD" {
        return Err(GitError::CurrentBranchNotFound);
    }
    Ok(branch)
}
/// Validate that a branch exists in the repository.
pub fn validate_branch_exists(project_path: &Path, branch: &str) -> Result<bool, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{branch}")])
        .current_dir(project_path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;
    if !output.status.success() {
        let output_remote = Command::new("git")
            .args([
                "rev-parse",
                "--verify",
                &format!("refs/remotes/origin/{branch}"),
            ])
            .current_dir(project_path)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .map_err(|e| GitError::CommandError(e.to_string()))?;
        return Ok(output_remote.status.success());
    }
    Ok(output.status.success())
}
/// Check if the current directory is a git repository.
#[must_use]
pub fn is_git_repository(project_path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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
