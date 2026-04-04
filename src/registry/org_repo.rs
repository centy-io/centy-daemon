//! Org-wide repo discovery helper.
//!
//! The org-wide `.centy` repo is a tracked project whose path ends with
//! `/.centy`.  Discovery is purely convention-based: no config flag is needed.
use super::storage::read_registry;
use super::RegistryError;
use std::path::Path;

/// Given a `project_path`, return the org repo's root path if one is tracked
/// for the same organization.
///
/// Algorithm:
/// 1. Resolve the current project's organization slug from the registry.
/// 2. Scan all tracked projects for one in the same org whose path ends in
///    `/.centy`.
/// 3. Return that path, or `None` if not found.
///
/// No caching.  No config fields.  No filesystem heuristics.
pub async fn find_org_repo(project_path: &str) -> Result<Option<String>, RegistryError> {
    let registry = read_registry().await?;

    let path = Path::new(project_path);
    let canonical = path.canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    // Resolve the org slug for the requesting project.
    let Some(org_slug) = registry
        .projects
        .get(&canonical)
        .or_else(|| registry.projects.get(project_path))
        .and_then(|p| p.organization_slug.as_deref())
        .map(str::to_owned)
    else {
        return Ok(None);
    };

    // Find a tracked project in the same org whose path ends with `/.centy`.
    for (tracked_path, tracked) in &registry.projects {
        if tracked.organization_slug.as_deref() == Some(&org_slug)
            && (tracked_path.ends_with("/.centy") || tracked_path == ".centy")
        {
            return Ok(Some(tracked_path.clone()));
        }
    }

    Ok(None)
}
