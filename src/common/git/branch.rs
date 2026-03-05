use std::path::Path;
use std::process::Command;

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
