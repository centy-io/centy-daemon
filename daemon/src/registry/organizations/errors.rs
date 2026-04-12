use crate::registry::RegistryError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrganizationError {
    #[error("Organization already exists: {0}")]
    AlreadyExists(String),

    #[error("Organization not found: {0}")]
    NotFound(String),

    #[error("Organization has {0} projects. Reassign or remove them first.")]
    HasProjects(u32),

    #[error("Invalid slug: {0}")]
    InvalidSlug(String),

    #[error("Project name '{project_name}' already exists in organization '{org_slug}'")]
    DuplicateNameInOrganization {
        project_name: String,
        org_slug: String,
    },

    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
