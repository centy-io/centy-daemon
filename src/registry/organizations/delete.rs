use super::errors::OrganizationError;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::utils::now_iso;
use tracing::info;

/// Delete an organization (fails if it has projects assigned)
pub async fn delete_organization(slug: &str) -> Result<(), OrganizationError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    if !registry.organizations.contains_key(slug) {
        return Err(OrganizationError::NotFound(slug.to_string()));
    }

    // Check for projects using this org
    let project_count = registry
        .projects
        .values()
        .filter(|p| p.organization_slug.as_deref() == Some(slug))
        .count() as u32;

    if project_count > 0 {
        return Err(OrganizationError::HasProjects(project_count));
    }

    registry.organizations.remove(slug);
    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    info!("Deleted organization: {}", slug);

    Ok(())
}
