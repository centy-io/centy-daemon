use thiserror::Error;
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Not a git repository")]
    NotGitRepository,
    #[error("Git error: {0}")]
    Git2Error(String),
    #[error("Failed to detect current branch")]
    CurrentBranchNotFound,
    #[error("Remote '{0}' not found")]
    RemoteNotFound(String),
}
