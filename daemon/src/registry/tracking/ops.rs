use super::super::inference::try_auto_assign_organization;
use super::super::storage::{get_lock, read_registry, write_registry_unlocked};
use super::super::types::TrackedProject;
use super::super::RegistryError;
pub use super::enrich_fn::enrich_project;
use crate::utils::now_iso;
use std::path::Path;
use tracing::warn;
/// Track a project access - called on any RPC operation
pub async fn track_project(project_path: &str) -> Result<(), RegistryError> {
    let path = Path::new(project_path);
    if super::super::ignore::is_ignored_path(path) {
        return Ok(());
    }
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
