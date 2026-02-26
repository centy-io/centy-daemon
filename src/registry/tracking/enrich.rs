use super::super::organizations::sync_org_from_project;
use super::super::storage::read_registry;
use super::super::types::{ListProjectsOptions, ProjectInfo};
use super::super::RegistryError;
use super::enrich_fn::enrich_project;
use super::super::inference::try_auto_assign_organization;
use std::path::Path;
/// List all tracked projects, enriched with live data
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines, max_nesting_depth)]
pub async fn list_projects(opts: ListProjectsOptions<'_>) -> Result<Vec<ProjectInfo>, RegistryError> {
    let registry = read_registry().await?;
    let mut projects = Vec::new();
    let mut ungrouped_paths: Vec<String> = Vec::new();
    for (path, tracked) in &registry.projects {
        let project_path = Path::new(path);
        let path_exists = project_path.exists();
        if !opts.include_stale && !path_exists { continue; }
        if !opts.include_temp && super::super::ignore::is_ignored_path(project_path) { continue; }
        if !opts.include_archived && tracked.is_archived { continue; }
        if tracked.organization_slug.is_none() && path_exists { ungrouped_paths.push(path.clone()); }
        if let Some(org_slug) = opts.organization_slug {
            if tracked.organization_slug.as_deref() != Some(org_slug) { continue; }
        }
        if opts.ungrouped_only && tracked.organization_slug.is_some() { continue; }
        let org_name = tracked.organization_slug.as_ref()
            .and_then(|slug| registry.organizations.get(slug))
            .map(|org| org.name.clone());
        if tracked.organization_slug.is_some() && org_name.is_none() {
            let project_path = Path::new(path);
            if let Ok(Some(_synced_slug)) = sync_org_from_project(project_path).await {}
        }
        let info = enrich_project(path, tracked, org_name).await;
        if !opts.include_uninitialized && !info.initialized { continue; }
        projects.push(info);
    }
    projects.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
    if !ungrouped_paths.is_empty() {
        tokio::spawn(async move {
            for path in ungrouped_paths { let _ = try_auto_assign_organization(&path, None).await; }
        });
    }
    Ok(projects)
}
/// Get info for a specific project
pub async fn get_project_info(project_path: &str) -> Result<Option<ProjectInfo>, RegistryError> {
    let path = Path::new(project_path);
    let canonical_path = path.canonicalize().map_or_else(|_| project_path.to_string(), |p| p.to_string_lossy().to_string());
    let registry = read_registry().await?;
    let tracked = registry.projects.get(&canonical_path).or_else(|| registry.projects.get(project_path));
    match tracked {
        Some(tracked) => {
            let org_name = tracked.organization_slug.as_ref()
                .and_then(|slug| registry.organizations.get(slug))
                .map(|org| org.name.clone());
            Ok(Some(enrich_project(&canonical_path, tracked, org_name).await))
        }
        None => Ok(None),
    }
}
/// Get all projects belonging to an organization, optionally excluding a specific project.
pub async fn get_org_projects(org_slug: &str, exclude_path: Option<&str>) -> Result<Vec<ProjectInfo>, RegistryError> {
    let opts = ListProjectsOptions { include_stale: false, include_uninitialized: false, include_archived: false, organization_slug: Some(org_slug), ungrouped_only: false, include_temp: false };
    let mut projects = list_projects(opts).await?;
    if let Some(exclude) = exclude_path { projects.retain(|p| p.path != exclude); }
    projects.retain(|p| p.initialized);
    Ok(projects)
}
