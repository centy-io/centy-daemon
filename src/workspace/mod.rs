//! Workspace management via worktree-io.
pub mod data;
mod create;
mod standalone;
mod types;

pub use create::create_temp_workspace;
pub use standalone::{create_standalone_workspace, remove_workspace};
pub use types::{CreateWorkspaceOptions, WorkspaceError};
#[allow(unused_imports)]
pub use types::{
    CreateStandaloneWorkspaceOptions, CreateStandaloneWorkspaceResult, CreateWorkspaceResult,
};
