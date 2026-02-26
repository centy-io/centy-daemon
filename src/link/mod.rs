mod crud;
mod storage;

pub use crud::{create_link, delete_link, get_available_link_types, list_links, CreateLinkOptions, DeleteLinkOptions};
pub use storage::{read_links, write_links, LinksFile};

#[allow(unused_imports)]
pub use crud::{CreateLinkResult, DeleteLinkResult, LinkError, LinkTypeInfo};

use serde::{Deserialize, Serialize};

/// Target entity type for links
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TargetType { Issue, Doc }

impl TargetType {
    #[must_use]
    pub fn as_str(&self) -> &'static str { match self { Self::Issue => "issue", Self::Doc => "doc" } }
    #[must_use]
    pub fn folder_name(&self) -> &'static str { match self { Self::Issue => "issues", Self::Doc => "docs" } }
}

impl std::str::FromStr for TargetType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "issue" => Ok(Self::Issue),
            "doc" => Ok(Self::Doc),
            _ => Err(format!("Invalid target type: {s}")),
        }
    }
}

impl std::fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.as_str()) }
}

/// A link between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub target_id: String,
    pub target_type: TargetType,
    pub link_type: String,
    pub created_at: String,
}

impl Link {
    #[must_use]
    pub fn new(target_id: String, target_type: TargetType, link_type: String) -> Self {
        Self { target_id, target_type, link_type, created_at: crate::utils::now_iso() }
    }
}

/// Custom link type definition (for config.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomLinkTypeDefinition {
    pub name: String,
    pub inverse: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Built-in link types and their inverses
pub const BUILTIN_LINK_TYPES: &[(&str, &str)] = &[
    ("blocks", "blocked-by"), ("blocked-by", "blocks"),
    ("parent-of", "child-of"), ("child-of", "parent-of"),
    ("relates-to", "related-from"), ("related-from", "relates-to"),
    ("duplicates", "duplicated-by"), ("duplicated-by", "duplicates"),
];

/// Get the inverse of a link type
pub fn get_inverse_link_type(link_type: &str, custom_types: &[CustomLinkTypeDefinition]) -> Option<String> {
    for (name, inverse) in BUILTIN_LINK_TYPES {
        if *name == link_type { return Some((*inverse).to_string()); }
    }
    for custom in custom_types {
        if custom.name == link_type { return Some(custom.inverse.clone()); }
        if custom.inverse == link_type { return Some(custom.name.clone()); }
    }
    None
}

/// Check if a link type is valid (either built-in or custom)
pub fn is_valid_link_type(link_type: &str, custom_types: &[CustomLinkTypeDefinition]) -> bool {
    BUILTIN_LINK_TYPES.iter().any(|(name, _)| *name == link_type)
        || custom_types.iter().any(|c| c.name == link_type || c.inverse == link_type)
}

#[cfg(test)]
#[path = "link_tests_1.rs"]
mod tests_1;
#[cfg(test)]
#[path = "link_tests_2.rs"]
mod tests_2;
#[cfg(test)]
#[path = "link_tests_3.rs"]
mod tests_3;
