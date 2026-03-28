use crate::registry::organizations::slugify;
use crate::registry::types::ProjectRegistry;
use std::collections::HashMap;
use std::path::Path;
use tracing::warn;

/// A group of projects that share the same slug within an organization.
#[derive(Debug, Clone)]
pub struct DuplicateSlugGroup {
    pub org_slug: String,
    pub project_slug: String,
    pub project_paths: Vec<String>,
}

/// Extract the project slug from a filesystem path.
fn project_slug_from_path(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|name| slugify(&name.to_string_lossy()))
}

/// Warn if the given project has a slug that conflicts with another
/// project in the same organization.
pub fn warn_on_slug_conflict(registry: &ProjectRegistry, project_path: &str) {
    let Some(tracked) = registry.projects.get(project_path) else {
        return;
    };
    let Some(org_slug) = tracked.organization_slug.as_deref() else {
        return;
    };
    let Some(my_slug) = project_slug_from_path(project_path) else {
        return;
    };
    for (other_path, other_tracked) in &registry.projects {
        if other_path == project_path {
            continue;
        }
        if other_tracked.organization_slug.as_deref() != Some(org_slug) {
            continue;
        }
        let Some(other_slug) = project_slug_from_path(other_path) else {
            continue;
        };
        if other_slug == my_slug {
            warn!(
                org = org_slug,
                slug = my_slug,
                path_a = project_path,
                path_b = other_path.as_str(),
                "Duplicate project slug within organization - \
                 the web UI may return 404 for one of these projects",
            );
            return;
        }
    }
}

/// Scan all tracked projects and return groups that share the same
/// slug within the same organization.
#[must_use]
pub fn find_duplicate_slugs(registry: &ProjectRegistry) -> Vec<DuplicateSlugGroup> {
    // Key: (org_slug, project_slug) -> list of project paths
    let mut groups: HashMap<(String, String), Vec<String>> = HashMap::new();
    for (path, tracked) in &registry.projects {
        let Some(org_slug) = tracked.organization_slug.as_deref() else {
            continue;
        };
        let Some(proj_slug) = project_slug_from_path(path) else {
            continue;
        };
        groups
            .entry((org_slug.to_string(), proj_slug))
            .or_default()
            .push(path.clone());
    }
    groups
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(
            |((org_slug, project_slug), project_paths)| DuplicateSlugGroup {
                org_slug,
                project_slug,
                project_paths,
            },
        )
        .collect()
}
