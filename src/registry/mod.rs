mod ignore;
mod inference;
mod organizations;
mod storage;
mod tracking;
mod types;
mod validation;

pub use ignore::init_ignore_paths;
pub use inference::{
    infer_organization_from_remote, try_auto_assign_organization, OrgInferenceResult,
};
pub use organizations::{
    create_org_issue, create_organization, delete_org_issue, delete_organization,
    get_org_config, get_org_issue, get_org_issue_by_display_number, get_organization,
    list_org_issues, list_organizations, set_project_organization, update_org_config,
    update_org_issue, update_organization, ListOrgIssuesOptions, OrgConfigError,
    OrgCustomFieldDef, OrgIssue, OrgIssueError, OrganizationError, UpdateOrgIssueOptions,
};
#[allow(unused_imports)]
pub use tracking::{
    get_org_projects, get_project_info, list_projects, set_project_archived, set_project_favorite,
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
