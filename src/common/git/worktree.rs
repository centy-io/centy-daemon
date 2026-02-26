use super::branch::is_git_repository;
use super::error::GitError;
use std::path::Path;
use std::process::Command;
/// Create a detached git worktree at the target path.
/// # Deprecated
/// For new code, prefer using `crate::workspace::gwq_client::GwqClient::add_worktree_at_path`.
pub fn create_worktree(
    source_path: &Path,
    target_path: &Path,
    git_ref: &str,
) -> Result<(), GitError> {
    if !is_git_repository(source_path) { return Err(GitError::NotGitRepository); }
    let output = Command::new("git")
        .args(["worktree", "add", "--detach", &target_path.to_string_lossy(), git_ref])
        .current_dir(source_path)
        .env_remove("GIT_DIR").env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::WorktreeError(stderr.to_string()));
    }
    Ok(())
}
/// Remove a git worktree.
/// # Deprecated
/// For new code, prefer using `crate::workspace::gwq_client::GwqClient::remove_worktree_from_repo`.
pub fn remove_worktree(source_path: &Path, worktree_path: &Path) -> Result<(), GitError> {
    let output = Command::new("git")
        .args(["worktree", "remove", "--force", &worktree_path.to_string_lossy()])
        .current_dir(source_path)
        .env_remove("GIT_DIR").env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("is not a working tree") {
            return Err(GitError::WorktreeError(stderr.to_string()));
        }
    }
    Ok(())
}
/// Prune stale worktree references.
/// # Deprecated
/// For new code, prefer using `crate::workspace::gwq_client::GwqClient::prune`.
pub fn prune_worktrees(source_path: &Path) -> Result<(), GitError> {
    let output = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(source_path)
        .env_remove("GIT_DIR").env_remove("GIT_WORK_TREE")
        .output()
        .map_err(|e| GitError::CommandError(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::WorktreeError(stderr.to_string()));
    }
    Ok(())
}
