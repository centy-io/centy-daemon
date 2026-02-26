use super::error::GitError;
use std::path::Path;
use std::process::Command;
/// Get the origin remote URL from a git repository.
pub fn get_remote_origin_url(project_path: &Path) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
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
        return Err(GitError::RemoteNotFound("origin".to_string()));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| GitError::InvalidUtf8)
        .map(|s| s.trim().to_string())
}
