use serde::{Deserialize, Serialize};

/// Project metadata stored in .centy/project.json (version-controlled, shared with team)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    /// Project-scope custom title (visible to all users)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
