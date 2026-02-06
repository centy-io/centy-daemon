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

#[cfg(test)]
mod tests {
    use super::*;

    // --- Organization tests ---

    #[test]
    fn test_organization_serialization() {
        let org = Organization {
            name: "Acme Corp".to_string(),
            description: Some("Our company".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-06-15T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&org).expect("Should serialize");
        let deserialized: Organization = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.name, "Acme Corp");
        assert_eq!(deserialized.description, Some("Our company".to_string()));
    }

    #[test]
    fn test_organization_without_description() {
        let org = Organization {
            name: "Test".to_string(),
            description: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&org).expect("Should serialize");
        assert!(!json.contains("description"));
    }

    #[test]
    fn test_organization_camel_case() {
        let org = Organization {
            name: "Test".to_string(),
            description: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&org).expect("Should serialize");
        assert!(json.contains("createdAt"));
        assert!(json.contains("updatedAt"));
    }

    // --- ProjectOrganization tests ---

    #[test]
    fn test_project_organization_serialization() {
        let po = ProjectOrganization {
            slug: "acme".to_string(),
            name: "Acme Corp".to_string(),
            description: Some("desc".to_string()),
        };

        let json = serde_json::to_string(&po).expect("Should serialize");
        let deserialized: ProjectOrganization =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.slug, "acme");
        assert_eq!(deserialized.name, "Acme Corp");
    }

    // --- TrackedProject tests ---

    #[test]
    fn test_tracked_project_serialization() {
        let tp = TrackedProject {
            first_accessed: "2024-01-01".to_string(),
            last_accessed: "2024-06-15".to_string(),
            is_favorite: true,
            is_archived: false,
            organization_slug: Some("acme".to_string()),
            user_title: Some("My Project".to_string()),
        };

        let json = serde_json::to_string(&tp).expect("Should serialize");
        let deserialized: TrackedProject = serde_json::from_str(&json).expect("Should deserialize");
        assert!(deserialized.is_favorite);
        assert!(!deserialized.is_archived);
        assert_eq!(deserialized.organization_slug, Some("acme".to_string()));
        assert_eq!(deserialized.user_title, Some("My Project".to_string()));
    }

    #[test]
    fn test_tracked_project_defaults() {
        let json = r#"{"firstAccessed":"2024-01-01","lastAccessed":"2024-01-01"}"#;
        let tp: TrackedProject = serde_json::from_str(json).expect("Should deserialize");
        assert!(!tp.is_favorite);
        assert!(!tp.is_archived);
        assert!(tp.organization_slug.is_none());
        assert!(tp.user_title.is_none());
    }

    #[test]
    fn test_tracked_project_skip_serializing_none() {
        let tp = TrackedProject {
            first_accessed: "2024-01-01".to_string(),
            last_accessed: "2024-01-01".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: None,
            user_title: None,
        };

        let json = serde_json::to_string(&tp).expect("Should serialize");
        assert!(!json.contains("organizationSlug"));
        assert!(!json.contains("userTitle"));
    }

    // --- ProjectRegistry tests ---

    #[test]
    fn test_project_registry_new() {
        let reg = ProjectRegistry::new();
        assert_eq!(reg.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(!reg.updated_at.is_empty());
        assert!(reg.organizations.is_empty());
        assert!(reg.projects.is_empty());
    }

    #[test]
    fn test_project_registry_default() {
        let reg = ProjectRegistry::default();
        assert_eq!(reg.schema_version, 0);
        assert!(reg.projects.is_empty());
        assert!(reg.organizations.is_empty());
    }

    #[test]
    fn test_project_registry_serialization() {
        let reg = ProjectRegistry::new();
        let json = serde_json::to_string(&reg).expect("Should serialize");
        let deserialized: ProjectRegistry =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_current_schema_version() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 2);
    }

    // --- ListProjectsOptions tests ---

    #[test]
    fn test_list_projects_options_default() {
        let opts = ListProjectsOptions::default();
        assert!(!opts.include_stale);
        assert!(!opts.include_uninitialized);
        assert!(!opts.include_archived);
        assert!(opts.organization_slug.is_none());
        assert!(!opts.ungrouped_only);
        assert!(!opts.include_temp);
    }

    #[test]
    fn test_list_projects_options_with_org_filter() {
        let opts = ListProjectsOptions {
            organization_slug: Some("acme"),
            ..Default::default()
        };
        assert_eq!(opts.organization_slug, Some("acme"));
    }

    // --- ProjectInfo tests ---

    #[test]
    fn test_project_info_debug() {
        let info = ProjectInfo {
            path: "/home/user/project".to_string(),
            first_accessed: "2024-01-01".to_string(),
            last_accessed: "2024-06-15".to_string(),
            issue_count: 10,
            doc_count: 5,
            initialized: true,
            name: Some("my-project".to_string()),
            is_favorite: true,
            is_archived: false,
            organization_slug: None,
            organization_name: None,
            user_title: None,
            project_title: None,
        };

        let debug = format!("{info:?}");
        assert!(debug.contains("ProjectInfo"));
        assert!(debug.contains("my-project"));
    }

    // --- OrganizationInfo tests ---

    #[test]
    fn test_organization_info_debug() {
        let info = OrganizationInfo {
            slug: "acme".to_string(),
            name: "Acme Corp".to_string(),
            description: Some("desc".to_string()),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            project_count: 3,
        };

        let debug = format!("{info:?}");
        assert!(debug.contains("OrganizationInfo"));
        assert!(debug.contains("Acme Corp"));
        assert!(debug.contains('3'));
    }
}
