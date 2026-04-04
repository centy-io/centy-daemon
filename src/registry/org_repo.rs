//! Org repo discovery helper.
//!
//! Given a `project_path`, finds the org repo tracked for the same organization,
//! if any. An org repo is identified by its path ending with `/.centy`.
use crate::registry::storage::read_registry;
use crate::registry::RegistryError;
use std::path::Path;

/// Find the org repo path for the project's organization, if one is tracked.
///
/// Returns `Some(path)` if a project in the same org has a path ending with
/// `/.centy`.  Returns `None` if the project has no org or no org repo is
/// tracked.
pub async fn find_org_repo(project_path: &str) -> Result<Option<String>, RegistryError> {
    let registry = read_registry().await?;

    let canonical = Path::new(project_path).canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    let org_slug = match registry
        .projects
        .get(&canonical)
        .and_then(|p| p.organization_slug.as_deref())
    {
        Some(slug) => slug.to_string(),
        None => return Ok(None),
    };

    for (path, project) in &registry.projects {
        if path == &canonical {
            continue;
        }
        if project.organization_slug.as_deref() == Some(org_slug.as_str())
            && (path.ends_with("/.centy") || path == "/.centy")
        {
            return Ok(Some(path.clone()));
        }
    }

    Ok(None)
}

#[cfg(test)]
#[path = "org_repo_tests.rs"]
mod tests;
