use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Organization stored in global registry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Organization info for per-project `.centy/organization.json` file
/// This travels with the project when cloned
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOrganization {
    pub slug: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Stored in registry file (minimal - only timestamps and favorite/archived status)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedProject {
    pub first_accessed: String,
    pub last_accessed: String,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_slug: Option<String>,
    /// User-scope custom title (only visible to this user)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_title: Option<String>,
}

/// The global project registry stored in ~/.centy/projects.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRegistry {
    /// Schema version for future migrations (2 = organizations support)
    pub schema_version: u32,

    /// When the registry was last modified
    pub updated_at: String,

    /// Map of organization slug -> Organization
    #[serde(default)]
    pub organizations: HashMap<String, Organization>,

    /// Map of project path -> `TrackedProject` (timestamps only)
    pub projects: HashMap<String, TrackedProject>,
}

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

impl ProjectRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            updated_at: crate::utils::now_iso(),
            organizations: HashMap::new(),
            projects: HashMap::new(),
        }
    }
}

/// Returned by API (enriched with live data from disk)
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Absolute path to the project root
    pub path: String,

    /// When the project was first tracked
    pub first_accessed: String,

    /// When the project was last accessed via any RPC
    pub last_accessed: String,

    /// Number of issues in the project (fetched live)
    pub issue_count: u32,

    /// Number of docs in the project (fetched live)
    pub doc_count: u32,

    /// Whether the project has been initialized (fetched live)
    pub initialized: bool,

    /// Project name (directory name, fetched live)
    pub name: Option<String>,

    /// User-marked favorite status (stored in registry)
    pub is_favorite: bool,

    /// User-marked archived status (stored in registry)
    pub is_archived: bool,

    /// Organization slug (stored in registry)
    pub organization_slug: Option<String>,

    /// Organization name (resolved from registry, for display)
    pub organization_name: Option<String>,

    /// User-scope custom title (stored in registry, only visible to this user)
    pub user_title: Option<String>,

    /// Project-scope custom title (stored in .centy/project.json, visible to all)
    pub project_title: Option<String>,
}

/// Organization info returned by API (enriched with project count)
#[derive(Debug, Clone)]
pub struct OrganizationInfo {
    /// Unique slug identifier
    pub slug: String,

    /// Display name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// When the organization was created
    pub created_at: String,

    /// When the organization was last updated
    pub updated_at: String,

    /// Number of projects in this organization (computed)
    pub project_count: u32,
}

/// Options for listing projects
#[derive(Debug, Clone, Default)]
pub struct ListProjectsOptions<'a> {
    /// Include projects where path no longer exists
    pub include_stale: bool,
    /// Include projects without .centy manifest
    pub include_uninitialized: bool,
    /// Include archived projects
    pub include_archived: bool,
    /// Filter by organization slug
    pub organization_slug: Option<&'a str>,
    /// Only show projects without organization
    pub ungrouped_only: bool,
    /// Include projects in system temp directory (default: false)
    pub include_temp: bool,
}
