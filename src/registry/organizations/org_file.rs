use super::errors::OrganizationError;
use crate::registry::types::ProjectOrganization;
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;

/// Read the .centy/organization.json file from a project
pub async fn read_project_org_file(
    project_path: &Path,
) -> Result<Option<ProjectOrganization>, OrganizationError> {
    let centy_path = get_centy_path(project_path);
    let org_file_path = centy_path.join("organization.json");

    if !org_file_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&org_file_path).await?;
    let org: ProjectOrganization = serde_json::from_str(&content)?;

    Ok(Some(org))
}

/// Write the .centy/organization.json file
pub(super) async fn write_project_org_file(
    path: &Path,
    org: &ProjectOrganization,
) -> Result<(), OrganizationError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let content = serde_json::to_string_pretty(org)?;
    fs::write(path, content).await?;

    Ok(())
}
