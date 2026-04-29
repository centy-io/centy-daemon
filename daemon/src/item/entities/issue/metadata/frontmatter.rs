use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Frontmatter metadata for the YAML-based issue format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueFrontmatter {
    pub display_number: u32,
    pub status: String,
    pub priority: u32,
    pub created_at: String,
    /// Retained for on-disk compatibility with mdstore Frontmatter; not surfaced via API.
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub draft: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    /// Project slugs this item belongs to. Empty for single-project items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, String>,
}
