use crate::registry::RegistryError;
use thiserror::Error;

fn duplicate_slug_message(
    project_name: &str,
    org_slug: &str,
    existing_path: &str,
    is_stale: bool,
) -> String {
    let base = format!(
        "Project '{project_name}' has the same slug as '{existing_path}' \
         in organization '{org_slug}'"
    );
    if is_stale {
        format!(
            "{base}. The existing project appears stale \
             (missing .centy folder). Remove it with: \
             centy untrack project {existing_path}"
        )
    } else {
        base
    }
}

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

    #[error("{}", duplicate_slug_message(project_name, org_slug, existing_path, *is_stale))]
    DuplicateSlugInOrganization {
        project_name: String,
        org_slug: String,
        existing_path: String,
        is_stale: bool,
    },

    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
