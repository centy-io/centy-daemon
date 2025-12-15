mod inference;
mod organizations;
mod storage;
mod tracking;
mod types;

pub use inference::{infer_organization_from_remote, try_auto_assign_organization, OrgInferenceResult};
pub use organizations::{
    create_organization, delete_organization, get_organization, list_organizations,
    set_project_organization, update_organization,
};
#[allow(unused_imports)]
pub use tracking::{
    get_project_info, list_projects, set_project_archived, set_project_favorite,
    set_project_user_title, track_project, track_project_async, untrack_project,
};
#[allow(unused_imports)]
pub use types::{
    ListProjectsOptions, Organization, OrganizationInfo, ProjectInfo, ProjectOrganization,
    ProjectRegistry, TrackedProject, CURRENT_SCHEMA_VERSION,
};

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
