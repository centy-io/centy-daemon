use super::errors::OrganizationError;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::utils::now_iso;
use tracing::info;

/// Delete an organization. When `cascade` is true, all projects assigned to the org are
/// untracked first. When false, deletion fails if any projects are assigned.
pub async fn delete_organization(slug: &str, cascade: bool) -> Result<u32, OrganizationError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    if !registry.organizations.contains_key(slug) {
        return Err(OrganizationError::NotFound(slug.to_string()));
    }

    // Collect project paths belonging to this org
    let org_projects: Vec<String> = registry
        .projects
        .iter()
        .filter(|(_, p)| p.organization_slug.as_deref() == Some(slug))
        .map(|(path, _)| path.clone())
        .collect();

    let project_count = org_projects.len() as u32;

    if project_count > 0 && !cascade {
        return Err(OrganizationError::HasProjects(project_count));
    }

    // Untrack all org projects inline (within the same lock) when cascading
    for path in &org_projects {
        registry.projects.remove(path);
    }

    registry.organizations.remove(slug);
    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    info!(
        "Deleted organization: {} (untracked {} project(s))",
        slug, project_count
    );

    Ok(project_count)
}
