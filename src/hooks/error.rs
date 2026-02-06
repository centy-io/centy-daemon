use thiserror::Error;

#[derive(Error, Debug)]
pub enum HookError {
    #[error("Pre-hook '{pattern}' failed with exit code {exit_code}: {stderr}")]
    PreHookFailed {
        pattern: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("Hook '{pattern}' timed out after {timeout_secs}s")]
    Timeout { pattern: String, timeout_secs: u64 },

    #[error("Hook execution error: {0}")]
    ExecutionError(String),

    #[error("Invalid hook pattern: {0}")]
    InvalidPattern(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
