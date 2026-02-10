use super::errors::OrganizationError;
use super::org_file::read_project_org_file;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::registry::types::Organization;
use crate::utils::now_iso;
use std::path::Path;
use tracing::info;

/// Sync an organization from a project's .centy/organization.json file.
/// Called when a project is first accessed (e.g., after cloning a repo).
/// If the org doesn't exist globally, it's auto-imported.
pub async fn sync_org_from_project(
    project_path: &Path,
) -> Result<Option<Organization>, OrganizationError> {
    let project_org = match read_project_org_file(project_path).await? {
        Some(org) => org,
        None => return Ok(None),
    };

    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    // Check if org already exists
    if let Some(existing) = registry.organizations.get(&project_org.slug) {
        return Ok(Some(existing.clone()));
    }

    // Auto-import the organization
    info!(
        "Auto-importing organization '{}' from project {}",
        project_org.slug,
        project_path.display()
    );

    let now = now_iso();
    let org = Organization {
        name: project_org.name,
        description: project_org.description,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    registry
        .organizations
        .insert(project_org.slug.clone(), org.clone());

    // Also link the project to this org
    let canonical_path = project_path.canonicalize().map_or_else(
        |_| project_path.to_string_lossy().to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    if let Some(project) = registry.projects.get_mut(&canonical_path) {
        project.organization_slug = Some(project_org.slug);
    }

    registry.updated_at = now;
    write_registry_unlocked(&registry).await?;

    Ok(Some(org))
}
