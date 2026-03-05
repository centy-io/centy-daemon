use thiserror::Error;
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Not a git repository")]
    NotGitRepository,
    #[error("Failed to execute git command: {0}")]
    CommandError(String),
    #[error("Git command output was not valid UTF-8")]
    InvalidUtf8,
    #[error("Remote '{0}' not found")]
    RemoteNotFound(String),
}
