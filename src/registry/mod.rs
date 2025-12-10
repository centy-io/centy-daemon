mod storage;
mod tracking;
mod types;

#[allow(unused_imports)]
pub use tracking::{
    get_project_info, list_projects, set_project_archived, set_project_favorite,
    track_project, track_project_async, untrack_project,
};
#[allow(unused_imports)]
pub use types::{ProjectInfo, ProjectRegistry, TrackedProject};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to determine home directory")]
    HomeDirNotFound,

    #[error("Project not found in registry: {0}")]
    ProjectNotFound(String),
}
