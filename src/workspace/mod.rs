//! Temporary workspace management for opening issues in VS Code.
//!
//! This module provides functionality to:
//! - Create temporary git worktrees for isolated issue work
//! - Set up VS Code with auto-running agent tasks
//! - Track and cleanup workspaces with TTL-based expiration
//!
//! ## Module Structure
//!
//! ### Core Components
//! - `gwq_client`: Wrapper for the gwq CLI tool (git worktree management)
//! - `metadata`: Centy-specific workspace metadata (TTL, issue binding, etc.)
//! - `orchestrator`: Main workspace creation orchestration (replaces `create`)
//! - `cleanup`: Workspace cleanup and expiration handling
//!
//! ### Supporting Components
//! - `editor`: Editor type and launching (value object)
//! - `path`: Workspace path generation and sanitization (value objects)
//! - `data`: Data copying for workspace setup (domain service)
//! - `storage`: Legacy registry persistence (infrastructure) - kept for backwards compatibility
//! - `types`: Shared type definitions
//! - `vscode`: VS Code configuration setup (infrastructure)
//! - `terminal`: Terminal launching (infrastructure)
//!
//! ## gwq Integration
//!
//! This module uses [gwq](https://github.com/d-kuro/gwq) for git worktree management.
//! gwq is bundled with centy and provides:
//! - Worktree creation/removal
//! - Worktree listing (JSON output)
//! - Worktree pruning
//!
//! Centy-specific metadata (TTL, issue binding, agent info) is stored separately
//! in `~/.centy/workspace-metadata.json`.

pub mod cleanup;
pub mod create; // Keep for backwards compatibility during transition
pub mod data;
pub mod editor;
pub mod gwq_client;
pub mod metadata;
pub mod orchestrator;
pub mod path;
pub mod storage;
pub mod terminal;
pub mod types;
pub mod vscode;

// Re-export from orchestrator (the new implementation)
#[allow(unused_imports)]
pub use cleanup::{cleanup_expired_workspaces, cleanup_workspace, CleanupResult};
#[allow(unused_imports)]
pub use editor::EditorType;
#[allow(unused_imports)]
pub use orchestrator::{
    create_standalone_workspace, create_temp_workspace, CreateStandaloneWorkspaceOptions,
    CreateStandaloneWorkspaceResult, CreateWorkspaceOptions, CreateWorkspaceResult,
};
#[allow(unused_imports)]
pub use path::{
    calculate_expires_at, generate_standalone_workspace_path, generate_workspace_path,
    sanitize_project_name, sanitize_workspace_name,
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

// Re-export gwq types for convenience
#[allow(unused_imports)]
pub use gwq_client::{GwqClient, GwqError, GwqWorktree};

// Re-export metadata types
#[allow(unused_imports)]
pub use metadata::{MetadataRegistry, WorkspaceMetadata};

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

    #[error("Source project not found: {0}")]
    SourceProjectNotFound(String),
}
