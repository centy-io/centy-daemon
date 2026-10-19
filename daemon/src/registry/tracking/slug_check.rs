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

/// Warn if the given project shares a slug with another project in the same
/// organization. Reuses [`find_duplicate_slugs`], so detection lives in one place.
pub fn warn_on_slug_conflict(registry: &ProjectRegistry, project_path: &str) {
    for group in find_duplicate_slugs(registry) {
        if group.project_paths.iter().any(|p| p == project_path) {
            warn!(
                org = group.org_slug.as_str(),
                slug = group.project_slug.as_str(),
                paths = ?group.project_paths,
                "Duplicate project slug within organization - \
                 the web UI may return 404 for one of these projects",
            );
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
