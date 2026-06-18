use super::super::storage::{get_lock, read_registry, write_registry_unlocked};
use super::super::types::ProjectInfo;
use super::super::RegistryError;
use super::enrich_fn::enrich_project;
use crate::utils::now_iso;
use std::path::Path;

#[cfg(test)]
#[path = "set_ops_tests.rs"]
mod tests;

/// Set the favorite status for a project
pub async fn set_project_favorite(
    project_path: &str,
    is_favorite: bool,
) -> Result<ProjectInfo, RegistryError> {
    let canonical_path = Path::new(project_path).canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    let _guard = get_lock().lock().await;
    let mut registry = read_registry().await?;
    let tracked = if let Some(t) = registry.projects.get_mut(canonical_path.as_str()) {
        t
    } else if let Some(t) = registry.projects.get_mut(project_path) {
        t
    } else {
        return Err(RegistryError::ProjectNotFound(project_path.to_string()));
    };
    tracked.is_favorite = is_favorite;
    let tracked_clone = tracked.clone();
    let org_name = tracked_clone
        .organization_slug
        .as_ref()
        .and_then(|slug| registry.organizations.get(slug))
        .map(|org| org.name.clone());
    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;
    Ok(enrich_project(&canonical_path, &tracked_clone, org_name).await)
}
/// Set the archived status for a project
pub async fn set_project_archived(
    project_path: &str,
    is_archived: bool,
) -> Result<ProjectInfo, RegistryError> {
    let canonical_path = Path::new(project_path).canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    let _guard = get_lock().lock().await;
    let mut registry = read_registry().await?;
    let tracked = if let Some(t) = registry.projects.get_mut(canonical_path.as_str()) {
        t
    } else if let Some(t) = registry.projects.get_mut(project_path) {
        t
    } else {
        return Err(RegistryError::ProjectNotFound(project_path.to_string()));
    };
    tracked.is_archived = is_archived;
    let tracked_clone = tracked.clone();
    let org_name = tracked_clone
        .organization_slug
        .as_ref()
        .and_then(|slug| registry.organizations.get(slug))
        .map(|org| org.name.clone());
    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;
    Ok(enrich_project(&canonical_path, &tracked_clone, org_name).await)
}
/// Set the user-scope custom title for a project
pub async fn set_project_user_title(
    project_path: &str,
    title: Option<String>,
) -> Result<ProjectInfo, RegistryError> {
    let canonical_path = Path::new(project_path).canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    let _guard = get_lock().lock().await;
    let mut registry = read_registry().await?;
    let tracked = if let Some(t) = registry.projects.get_mut(canonical_path.as_str()) {
        t
    } else if let Some(t) = registry.projects.get_mut(project_path) {
        t
    } else {
        return Err(RegistryError::ProjectNotFound(project_path.to_string()));
    };
    tracked.user_title = title.filter(|t| !t.is_empty());
    let tracked_clone = tracked.clone();
    let org_name = tracked_clone
        .organization_slug
        .as_ref()
        .and_then(|slug| registry.organizations.get(slug))
        .map(|org| org.name.clone());
    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;
    Ok(enrich_project(&canonical_path, &tracked_clone, org_name).await)
}
