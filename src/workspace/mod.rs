//! Temporary workspace management for opening issues in VS Code.
//!
//! This module provides functionality to:
//! - Create temporary git worktrees for isolated issue work
//! - Set up VS Code with auto-running agent tasks
//! - Track and cleanup workspaces with TTL-based expiration

pub mod cleanup;
pub mod create;
pub mod storage;
pub mod terminal;
pub mod types;
pub mod vscode;

#[allow(unused_imports)]
pub use cleanup::{cleanup_expired_workspaces, cleanup_workspace, CleanupResult};
#[allow(unused_imports)]
pub use create::{create_temp_workspace, CreateWorkspaceOptions, CreateWorkspaceResult};
#[allow(unused_imports)]
pub use storage::{
    add_workspace, find_workspace_for_issue, get_workspace, list_workspaces, read_registry,
    remove_workspace, update_workspace_expiration, write_registry,
};
#[allow(unused_imports)]
pub use types::{TempWorkspaceEntry, WorkspaceRegistry, DEFAULT_TTL_HOURS};
#[allow(unused_imports)]
pub use vscode::{open_vscode, setup_vscode_config};
#[allow(unused_imports)]
pub use terminal::{is_terminal_available, open_terminal_with_agent};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Source is not a git repository")]
    NotGitRepository,

    #[error("Git error: {0}")]
    GitError(String),

    #[error("VS Code failed to open: {0}")]
    VscodeError(String),

    #[error("Terminal failed to open: {0}")]
    TerminalError(String),

    #[error("Issue error: {0}")]
    IssueError(#[from] crate::issue::IssueCrudError),

    #[error("Config error: {0}")]
    ConfigError(#[from] crate::llm::LocalConfigError),

    #[error("Source project not found: {0}")]
    SourceProjectNotFound(String),
}
