#![allow(unknown_lints, max_lines_per_file)]
use super::super::inference::try_auto_assign_organization;
use super::super::storage::{get_lock, read_registry, write_registry_unlocked};
use super::super::types::{ProjectInfo, TrackedProject};
use super::super::RegistryError;
pub use super::enrich_fn::enrich_project;
use crate::utils::now_iso;
use std::path::Path;
use tracing::warn;
/// Track a project access - called on any RPC operation
pub async fn track_project(project_path: &str) -> Result<(), RegistryError> {
    let path = Path::new(project_path);
    let canonical_path = path.canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    let needs_org_inference: bool;
    {
        let _guard = get_lock().lock().await;
        let mut registry = read_registry().await?;
        let now = now_iso();
        if let Some(entry) = registry.projects.get_mut(&canonical_path) {
            entry.last_accessed.clone_from(&now);
            needs_org_inference = entry.organization_slug.is_none();
        } else {
            registry.projects.insert(
                canonical_path.clone(),
                TrackedProject {
                    first_accessed: now.clone(),
                    last_accessed: now.clone(),
                    is_favorite: false,
                    is_archived: false,
                    organization_slug: None,
                    user_title: None,
                },
            );
            needs_org_inference = true;
        }
        registry.updated_at = now;
        write_registry_unlocked(&registry).await?;
    }
    if needs_org_inference {
        let path_for_inference = canonical_path;
        tokio::spawn(async move {
            let _ = try_auto_assign_organization(&path_for_inference, None).await;
        });
    }
    Ok(())
}
/// Track project access asynchronously (fire-and-forget)
pub fn track_project_async(project_path: String) {
    tokio::spawn(async move {
        if let Err(e) = track_project(&project_path).await {
            warn!("Failed to track project {}: {}", project_path, e);
        }
    });
}
/// Remove a project from tracking
pub async fn untrack_project(project_path: &str) -> Result<(), RegistryError> {
    let path = Path::new(project_path);
    let canonical_path = path.canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    let _guard = get_lock().lock().await;
    let mut registry = read_registry().await?;
    if registry.projects.remove(&canonical_path).is_none()
        && registry.projects.remove(project_path).is_none()
    {
        return Err(RegistryError::ProjectNotFound(project_path.to_string()));
    }
    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;
    Ok(())
}
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
