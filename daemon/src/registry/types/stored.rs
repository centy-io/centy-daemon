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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOrganization {
    pub slug: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
/// Stored in registry file (minimal)
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_title: Option<String>,
}
/// The global project registry stored in ~/.centy/projects.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRegistry {
    pub schema_version: u32,
    pub updated_at: String,
    #[serde(default)]
    pub organizations: HashMap<String, Organization>,
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
