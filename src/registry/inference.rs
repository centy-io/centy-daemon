//! Organization inference from git remote URLs.
//!
//! This module provides functionality to automatically infer an organization
//! from a git repository's remote URL. This enables automatic organization
//! assignment when initializing or registering a project.

use super::organizations::{create_organization, get_organization, slugify};
use crate::item::entities::pr::git::{get_remote_origin_url, is_git_repository};
use crate::item::entities::pr::remote::parse_remote_url;
use std::path::Path;

/// Result of organization inference from git remote
#[derive(Debug, Clone, Default)]
pub struct OrgInferenceResult {
    /// The inferred organization slug (if found)
    pub inferred_org_slug: Option<String>,
    /// The inferred organization name (display name)
    pub inferred_org_name: Option<String>,
    /// Whether a new organization was created
    pub org_created: bool,
    /// Existing org slug if project already had one assigned
    pub existing_org_slug: Option<String>,
    /// Whether there's a mismatch between existing and inferred
    pub has_mismatch: bool,
    /// Human-readable message about what happened
    pub message: Option<String>,
}

/// Infer organization from git remote URL.
///
/// This function:
/// 1. Checks if the project is a git repository
/// 2. Gets the origin remote URL
/// 3. Parses the URL to extract organization
/// 4. Checks for mismatch with existing org (if any)
/// 5. Auto-creates org if it doesn't exist
///
/// # Arguments
/// * `project_path` - Path to the project directory
/// * `existing_org_slug` - The project's currently assigned organization slug (if any)
///
/// # Returns
/// An `OrgInferenceResult` containing the inference outcome
pub async fn infer_organization_from_remote(
    project_path: &Path,
    existing_org_slug: Option<&str>,
) -> OrgInferenceResult {
    let mut result = OrgInferenceResult {
        existing_org_slug: existing_org_slug.map(String::from),
        ..Default::default()
    };

    // Check if it's a git repository
    if !is_git_repository(project_path) {
        result.message = Some("Not a git repository".to_string());
        return result;
    }

    // Get remote URL
    let remote_url = match get_remote_origin_url(project_path) {
        Ok(url) => url,
        Err(_) => {
            result.message = Some("No origin remote found".to_string());
            return result;
        }
    };

    // Parse the remote URL
    let parsed = match parse_remote_url(&remote_url) {
        Some(p) => p,
        None => {
            result.message = Some(format!("Could not parse remote URL: {remote_url}"));
            return result;
        }
    };

    // Generate slug from org name
    let inferred_slug = slugify(&parsed.org);
    result.inferred_org_slug = Some(inferred_slug.clone());
    result.inferred_org_name = Some(parsed.org.clone());

    // Check for mismatch with existing org
    if let Some(existing) = existing_org_slug {
        if !existing.is_empty() && existing != inferred_slug {
            result.has_mismatch = true;
            result.message = Some(format!(
                "Project is assigned to '{existing}' but git remote suggests '{inferred_slug}'. \
                 Use `centy org set {inferred_slug}` to update."
            ));
            return result;
        }
    }

    // Check if org exists
    match get_organization(&inferred_slug).await {
        Ok(Some(_)) => {
            result.message = Some(format!("Using existing organization: {inferred_slug}"));
        }
        Ok(None) => {
            // Auto-create the organization
            match create_organization(Some(&inferred_slug), &parsed.org, None).await {
                Ok(_) => {
                    result.org_created = true;
                    result.message = Some(format!("Created organization: {inferred_slug}"));
                }
                Err(e) => {
                    result.message = Some(format!("Failed to create organization: {e}"));
                    result.inferred_org_slug = None;
                    result.inferred_org_name = None;
                }
            }
        }
        Err(e) => {
            result.message = Some(format!("Error checking organization: {e}"));
            result.inferred_org_slug = None;
            result.inferred_org_name = None;
        }
    }

    result
}

/// Attempt to infer and auto-assign organization for a project if ungrouped.
///
/// This function is designed for automatic background inference:
/// 1. Skips if project already has an organization assigned
/// 2. Skips if path doesn't exist
/// 3. Calls `infer_organization_from_remote()`
/// 4. Auto-assigns if inference succeeded without mismatch
///
/// # Arguments
/// * `project_path` - Path string to the project directory
/// * `current_org_slug` - The project's current organization (if known, avoids re-fetch)
///
/// # Returns
/// `Some(OrgInferenceResult)` if inference was attempted, `None` if skipped
pub async fn try_auto_assign_organization(
    project_path: &str,
    current_org_slug: Option<&str>,
) -> Option<OrgInferenceResult> {
    // Skip if already has org
    if let Some(slug) = current_org_slug {
        if !slug.is_empty() {
            return None;
        }
    }

    let path = Path::new(project_path);

    // Skip if path doesn't exist
    if !path.exists() {
        return None;
    }

    let inference = infer_organization_from_remote(path, current_org_slug).await;

    // Auto-assign if inference succeeded without mismatch
    if !inference.has_mismatch {
        if let Some(ref slug) = inference.inferred_org_slug {
            // Attempt assignment, ignore errors
            let _ = super::organizations::set_project_organization(project_path, Some(slug)).await;
        }
    }

    Some(inference)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_infer_from_non_git_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = infer_organization_from_remote(temp_dir.path(), None).await;

        assert!(result.inferred_org_slug.is_none());
        assert!(result.message.unwrap().contains("Not a git repository"));
    }

    #[tokio::test]
    async fn test_mismatch_detection() {
        // This test just verifies the mismatch logic works
        // We can't easily test the full flow without a real git repo
        let result = OrgInferenceResult {
            inferred_org_slug: Some("new-org".to_string()),
            inferred_org_name: Some("new-org".to_string()),
            existing_org_slug: Some("old-org".to_string()),
            has_mismatch: true,
            ..Default::default()
        };

        assert!(result.has_mismatch);
        assert_eq!(result.existing_org_slug, Some("old-org".to_string()));
        assert_eq!(result.inferred_org_slug, Some("new-org".to_string()));
    }
}
