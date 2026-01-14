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
pub use create::{
    create_standalone_workspace, create_temp_workspace, CreateStandaloneWorkspaceOptions,
    CreateStandaloneWorkspaceResult, CreateWorkspaceOptions, CreateWorkspaceResult, EditorType,
};
#[allow(unused_imports)]
pub use storage::{
    add_workspace, find_standalone_workspace, find_workspace_for_issue, get_workspace,
    list_workspaces, read_registry, remove_workspace, update_workspace_expiration, write_registry,
};
#[allow(unused_imports)]
pub use terminal::{is_terminal_available, open_terminal, open_terminal_with_agent};
#[allow(unused_imports)]
pub use types::{TempWorkspaceEntry, WorkspaceRegistry, DEFAULT_TTL_HOURS};
#[allow(unused_imports)]
pub use vscode::{open_vscode, setup_vscode_config};

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
    #[allow(dead_code)] // Used on macOS and Windows only
    TerminalError(String),

    #[error("No terminal emulator found")]
    #[allow(dead_code)] // Used on Linux and unsupported platforms only
    TerminalNotFound,

    #[error("Issue error: {0}")]
    IssueError(#[from] crate::item::entities::issue::IssueCrudError),

    #[error("Config error: {0}")]
    ConfigError(#[from] crate::llm::LocalConfigError),

    #[error("Source project not found: {0}")]
    SourceProjectNotFound(String),
}
