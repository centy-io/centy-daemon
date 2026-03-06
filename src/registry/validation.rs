//! Validation service for registry operations
use super::organizations::slugify;
use super::organizations::OrganizationError;
use super::types::ProjectRegistry;
use crate::utils::get_centy_path;
use std::path::Path;

/// Validation service for project and organization constraints
pub struct ValidationService;

impl ValidationService {
    /// Check that no other project in the same org produces the same slug.
    ///
    /// The slug is derived via `slugify` from the directory name, which is
    /// what the web UI uses in URLs. Two folders like `my_app` and `my-app`
    /// both slugify to `my-app`, causing a 404 in the web UI.
    ///
    /// When a conflict is found against a stale project (missing `.centy`
    /// folder), the error message suggests removing it via `centy untrack`.
    pub fn validate_unique_project_slug(
        registry: &ProjectRegistry,
        org_slug: &str,
        current_project_path: &str,
        project_name: &str,
    ) -> Result<(), OrganizationError> {
        let new_slug = slugify(project_name);
        for (path, tracked) in &registry.projects {
            if path == current_project_path {
                continue;
            }
            if tracked.organization_slug.as_deref() != Some(org_slug) {
                continue;
            }
            let Some(existing_name) = Self::extract_project_name(path) else {
                continue;
            };
            if slugify(&existing_name) == new_slug {
                let is_stale = !get_centy_path(Path::new(path)).exists();
                return Err(OrganizationError::DuplicateSlugInOrganization {
                    project_name: project_name.to_string(),
                    org_slug: org_slug.to_string(),
                    existing_path: path.clone(),
                    is_stale,
                });
            }
        }
        Ok(())
    }

    fn extract_project_name(path: &str) -> Option<String> {
        Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
    }
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;
