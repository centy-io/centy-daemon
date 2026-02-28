use super::super::inference::try_auto_assign_organization;
use super::super::organizations::sync_org_from_project;
use super::super::storage::read_registry;
use super::super::types::{ListProjectsOptions, ProjectInfo};
use super::super::RegistryError;
use super::enrich_fn::enrich_project;
use std::path::Path;
/// List all tracked projects, enriched with live data
pub async fn list_projects(
    opts: ListProjectsOptions<'_>,
) -> Result<Vec<ProjectInfo>, RegistryError> {
    let registry = read_registry().await?;
    let mut projects = Vec::new();
    let mut ungrouped_paths: Vec<String> = Vec::new();
    for (path, tracked) in &registry.projects {
        let project_path = Path::new(path);
        let path_exists = project_path.exists();
        if !opts.include_stale && !path_exists {
            continue;
        }
        if !opts.include_temp && super::super::ignore::is_ignored_path(project_path) {
            continue;
        }
        if !opts.include_archived && tracked.is_archived {
            continue;
        }
        if tracked.organization_slug.is_none() && path_exists {
            ungrouped_paths.push(path.clone());
        }
        if opts
            .organization_slug
            .is_some_and(|s| tracked.organization_slug.as_deref() != Some(s))
        {
            continue;
        }
        if opts.ungrouped_only && tracked.organization_slug.is_some() {
            continue;
        }
        let org_name = tracked
            .organization_slug
            .as_ref()
            .and_then(|slug| registry.organizations.get(slug))
            .map(|org| org.name.clone());
        if tracked.organization_slug.is_some() && org_name.is_none() {
            let _ = sync_org_from_project(Path::new(path)).await;
        }
        let info = enrich_project(path, tracked, org_name).await;
        if !opts.include_uninitialized && !info.initialized {
            continue;
        }
        projects.push(info);
    }
    projects.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
    if !ungrouped_paths.is_empty() {
        spawn_auto_assign(ungrouped_paths);
    }
    Ok(projects)
}
/// Get all projects belonging to an organization, optionally excluding a specific project.
pub async fn get_org_projects(
    org_slug: &str,
    exclude_path: Option<&str>,
) -> Result<Vec<ProjectInfo>, RegistryError> {
    let opts = ListProjectsOptions {
        include_stale: false,
        include_uninitialized: false,
        include_archived: false,
        organization_slug: Some(org_slug),
        ungrouped_only: false,
        include_temp: false,
    };
    let mut projects = list_projects(opts).await?;
    if let Some(exclude) = exclude_path {
        projects.retain(|p| p.path != exclude);
    }
    projects.retain(|p| p.initialized);
    Ok(projects)
}
fn spawn_auto_assign(paths: Vec<String>) {
    tokio::spawn(async move {
        for path in paths {
            let _ = try_auto_assign_organization(&path, None).await;
        }
    });
}
