//! Validation service for registry operations

use super::organizations::OrganizationError;
use super::types::ProjectRegistry;
use std::path::Path;

/// Validation service for project and organization constraints
pub struct ValidationService;

impl ValidationService {
    /// Check if a project name is unique within an organization
    ///
    /// # Arguments
    /// * `registry` - The project registry to check against
    /// * `org_slug` - The organization slug to check within
    /// * `current_project_path` - Path of the current project (excluded from check for idempotency)
    /// * `project_name` - Name of the project to validate
    ///
    /// # Returns
    /// * `Ok(())` if the name is unique or this is an idempotent reassignment
    /// * `Err(OrganizationError::DuplicateNameInOrganization)` if a duplicate exists
    ///
    /// # Example
    /// ```ignore
    /// ValidationService::validate_unique_project_name(
    ///     &registry,
    ///     "my-org",
    ///     "/path/to/project",
    ///     "myapp"
    /// )?;
    /// ```
    pub fn validate_unique_project_name(
        registry: &ProjectRegistry,
        org_slug: &str,
        current_project_path: &str,
        project_name: &str,
    ) -> Result<(), OrganizationError> {
        // Normalize project name for case-insensitive comparison
        let project_name_normalized = Self::normalize_name(project_name);

        // Check all projects in the registry
        for (path, tracked) in &registry.projects {
            // Skip the current project (for idempotent operations)
            if path == current_project_path {
                continue;
            }

            // Only check projects in the same organization
            if tracked.organization_slug.as_deref() != Some(org_slug) {
                continue;
            }

            // Extract the directory name from the path
            if let Some(existing_name) = Self::extract_project_name(path) {
                let existing_name_normalized = Self::normalize_name(&existing_name);

                // Check for duplicate (case-insensitive)
                if existing_name_normalized == project_name_normalized {
                    return Err(OrganizationError::DuplicateNameInOrganization {
                        project_name: project_name.to_string(),
                        org_slug: org_slug.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Normalize a project name for comparison (lowercase, trimmed)
    fn normalize_name(name: &str) -> String {
        name.trim().to_lowercase()
    }

    /// Extract project name from a path (directory name)
    fn extract_project_name(path: &str) -> Option<String> {
        Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(ValidationService::normalize_name("MyApp"), "myapp");
        assert_eq!(ValidationService::normalize_name("  MyApp  "), "myapp");
        assert_eq!(ValidationService::normalize_name("MYAPP"), "myapp");
        assert_eq!(ValidationService::normalize_name("myapp"), "myapp");
    }

    #[test]
    fn test_extract_project_name() {
        assert_eq!(
            ValidationService::extract_project_name("/path/to/myapp"),
            Some("myapp".to_string())
        );
        assert_eq!(
            ValidationService::extract_project_name("/myapp"),
            Some("myapp".to_string())
        );
        assert_eq!(
            ValidationService::extract_project_name("myapp"),
            Some("myapp".to_string())
        );
    }
}
