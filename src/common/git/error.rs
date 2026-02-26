use thiserror::Error;
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Not a git repository")]
    NotGitRepository,
    #[error("Failed to execute git command: {0}")]
    CommandError(String),
    #[error("Branch '{0}' does not exist")]
    BranchNotFound(String),
    #[error("Failed to detect current branch")]
    CurrentBranchNotFound,
    #[error("Git command output was not valid UTF-8")]
    InvalidUtf8,
    #[error("Worktree error: {0}")]
    WorktreeError(String),
    #[error("Remote '{0}' not found")]
    RemoteNotFound(String),
}
