//! Validation service for registry operations
use super::organizations::OrganizationError;
use super::types::ProjectRegistry;
use std::path::Path;

/// Validation service for project and organization constraints
pub struct ValidationService;

impl ValidationService {
    /// Check if a project name is unique within an organization.
    ///
    /// # Example
    /// ```ignore
    /// ValidationService::validate_unique_project_name(&registry, "my-org", "/path/to/project", "myapp")?;
    /// ```
    pub fn validate_unique_project_name(
        registry: &ProjectRegistry,
        org_slug: &str,
        current_project_path: &str,
        project_name: &str,
    ) -> Result<(), OrganizationError> {
        let project_name_normalized = Self::normalize_name(project_name);
        for (path, tracked) in &registry.projects {
            if path == current_project_path {
                continue;
            }
            if tracked.organization_slug.as_deref() != Some(org_slug) {
                continue;
            }
            if let Some(existing_name) = Self::extract_project_name(path) {
                if Self::normalize_name(&existing_name) == project_name_normalized {
                    return Err(OrganizationError::DuplicateNameInOrganization {
                        project_name: project_name.to_string(),
                        org_slug: org_slug.to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn normalize_name(name: &str) -> String {
        name.trim().to_lowercase()
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
