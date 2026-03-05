//! Workspace management via worktree-io.
mod create;
pub mod data;
mod standalone;
mod types;

pub use create::create_temp_workspace;
pub use standalone::{create_standalone_workspace, remove_workspace};
#[allow(unused_imports)]
pub use types::{
    CreateStandaloneWorkspaceOptions, CreateStandaloneWorkspaceResult, CreateWorkspaceResult,
};
pub use types::{CreateWorkspaceOptions, WorkspaceError};
