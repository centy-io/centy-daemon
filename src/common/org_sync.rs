//! Cross-organization item synchronization traits and utilities.
//!
//! This module provides a generic abstraction for syncing items (issues, docs, etc.)
//! across all projects within an organization.

use async_trait::async_trait;
use std::path::Path;
use thiserror::Error;

use crate::registry::{get_org_projects, RegistryError};

/// Error types for org sync operations
#[derive(Error, Debug)]
pub enum OrgSyncError {
    #[error("Project has no organization")]
    NoOrganization,

    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Sync failed: {0}")]
    SyncFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Manifest error: {0}")]
    ManifestError(String),
}

/// Result of syncing an org item to a single project
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrgSyncResult {
    /// Path of the project that was synced to
    pub project_path: String,
    /// Whether the sync was successful
    pub success: bool,
    /// Error message if sync failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Trait for items that can be synced across organization projects.
///
/// Implementing this trait allows an item type (Issue, Doc, etc.) to be
/// automatically synced to all projects within an organization when
/// created or updated.
///
/// # Example
///
/// ```ignore
/// #[async_trait]
/// impl OrgSyncable for Issue {
///     fn item_id(&self) -> &str {
///         &self.id
///     }
///
///     fn is_org_item(&self) -> bool {
///         self.metadata.is_org_issue
///     }
///
///     fn org_slug(&self) -> Option<&str> {
///         self.metadata.org_slug.as_deref()
///     }
///
///     async fn sync_to_project(&self, target: &Path, org_slug: &str) -> Result<(), OrgSyncError> {
///         // Create/update the issue in the target project
///     }
/// }
/// ```
#[async_trait]
pub trait OrgSyncable: Sized + Send + Sync {
    /// The unique identifier for this item (e.g., UUID for issues, slug for docs)
    fn item_id(&self) -> &str;

    /// Whether this item is an organization-level item
    fn is_org_item(&self) -> bool;

    /// The organization slug, if this is an org item
    fn org_slug(&self) -> Option<&str>;

    /// Create or update this item in the target project.
    ///
    /// This method should NOT trigger recursive org sync (to prevent infinite loops).
    /// It should handle both creation (if item doesn't exist) and updating (if it does).
    async fn sync_to_project(
        &self,
        target_project: &Path,
        org_slug: &str,
    ) -> Result<(), OrgSyncError>;

    /// Sync an update to the target project, with optional old ID for renames.
    ///
    /// Default implementation just calls `sync_to_project`. Override this if
    /// your item type needs special handling for renames (e.g., doc slug changes).
    async fn sync_update_to_project(
        &self,
        target_project: &Path,
        org_slug: &str,
        _old_id: Option<&str>,
    ) -> Result<(), OrgSyncError> {
        self.sync_to_project(target_project, org_slug).await
    }
}

/// Orchestrate syncing an org item to all organization projects.
///
/// This function:
/// 1. Gets all projects in the organization (excluding the source project)
/// 2. Calls `sync_to_project` for each project
/// 3. Returns results for each project
///
/// # Arguments
///
/// * `item` - The item to sync (must implement `OrgSyncable`)
/// * `source_project_path` - Path of the project where the item was created/updated
///
/// # Returns
///
/// A vector of `OrgSyncResult` indicating success/failure for each project.
/// Returns empty vector if the item is not an org item.
pub async fn sync_to_org_projects<T: OrgSyncable>(
    item: &T,
    source_project_path: &Path,
) -> Vec<OrgSyncResult> {
    let org_slug = match item.org_slug() {
        Some(slug) => slug,
        None => return Vec::new(),
    };

    let source_path_str = source_project_path.to_string_lossy().to_string();

    // Get all other projects in the org
    let org_projects = match get_org_projects(org_slug, Some(&source_path_str)).await {
        Ok(projects) => projects,
        Err(e) => {
            // Return a single error result
            return vec![OrgSyncResult {
                project_path: "<registry>".to_string(),
                success: false,
                error: Some(format!("Failed to get org projects: {e}")),
            }];
        }
    };

    let mut results = Vec::new();

    for project in org_projects {
        let target_path = Path::new(&project.path);
        let result = item.sync_to_project(target_path, org_slug).await;

        results.push(OrgSyncResult {
            project_path: project.path.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        });
    }

    results
}

/// Orchestrate syncing an org item update to all organization projects.
///
/// Similar to `sync_to_org_projects`, but uses `sync_update_to_project` which
/// can handle special cases like ID/slug renames.
///
/// # Arguments
///
/// * `item` - The updated item to sync
/// * `source_project_path` - Path of the project where the update originated
/// * `old_id` - Optional old ID if the item's identifier changed (e.g., slug rename)
///
/// # Returns
///
/// A vector of `OrgSyncResult` indicating success/failure for each project.
/// Returns empty vector if the item is not an org item.
pub async fn sync_update_to_org_projects<T: OrgSyncable>(
    item: &T,
    source_project_path: &Path,
    old_id: Option<&str>,
) -> Vec<OrgSyncResult> {
    let org_slug = match item.org_slug() {
        Some(slug) => slug,
        None => return Vec::new(),
    };

    let source_path_str = source_project_path.to_string_lossy().to_string();

    // Get all other projects in the org
    let org_projects = match get_org_projects(org_slug, Some(&source_path_str)).await {
        Ok(projects) => projects,
        Err(e) => {
            // Return a single error result
            return vec![OrgSyncResult {
                project_path: "<registry>".to_string(),
                success: false,
                error: Some(format!("Failed to get org projects: {e}")),
            }];
        }
    };

    let mut results = Vec::new();

    for project in org_projects {
        let target_path = Path::new(&project.path);
        let result = item.sync_update_to_project(target_path, org_slug, old_id).await;

        results.push(OrgSyncResult {
            project_path: project.path.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_sync_result_serialization() {
        let result = OrgSyncResult {
            project_path: "/path/to/project".to_string(),
            success: true,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"project_path\""));
        assert!(json.contains("\"success\":true"));
        // error should be omitted when None
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_org_sync_result_with_error() {
        let result = OrgSyncResult {
            project_path: "/path/to/project".to_string(),
            success: false,
            error: Some("Failed to sync".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"Failed to sync\""));
    }
}
