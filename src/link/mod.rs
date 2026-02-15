mod crud;
mod storage;

pub use crud::{
    create_link, delete_link, get_available_link_types, list_links, CreateLinkOptions,
    DeleteLinkOptions,
};
pub use storage::{read_links, write_links, LinksFile};

// Re-export types that are part of public API (used in lib.rs)
// These are intentionally exported even if not used in the binary
#[allow(unused_imports)]
pub use crud::{CreateLinkResult, DeleteLinkResult, LinkError, LinkTypeInfo};

use serde::{Deserialize, Serialize};

/// Target entity type for links
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Issue,
    Doc,
}

impl TargetType {
    /// Convert to string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Issue => "issue",
            Self::Doc => "doc",
        }
    }

    /// Get the folder name for this entity type
    #[must_use]
    pub fn folder_name(&self) -> &'static str {
        match self {
            Self::Issue => "issues",
            Self::Doc => "docs",
        }
    }
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A link between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// The ID of the target entity (UUID for issues, slug for docs)
    pub target_id: String,
    /// The type of the target entity
    pub target_type: TargetType,
    /// The type of relationship (e.g., "blocks", "parent-of", "relates-to")
    pub link_type: String,
    /// ISO timestamp when the link was created
    pub created_at: String,
}

impl Link {
    /// Create a new link
    #[must_use]
    pub fn new(target_id: String, target_type: TargetType, link_type: String) -> Self {
        Self {
            target_id,
            target_type,
            link_type,
            created_at: crate::utils::now_iso(),
        }
    }
}

/// Custom link type definition (for config.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomLinkTypeDefinition {
    /// The name of the link type (e.g., "depends-on")
    pub name: String,
    /// The inverse link type (e.g., "dependency-of")
    pub inverse: String,
    /// Optional description of this link type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Built-in link types and their inverses
pub const BUILTIN_LINK_TYPES: &[(&str, &str)] = &[
    ("blocks", "blocked-by"),
    ("blocked-by", "blocks"),
    ("parent-of", "child-of"),
    ("child-of", "parent-of"),
    ("relates-to", "related-from"),
    ("related-from", "relates-to"),
    ("duplicates", "duplicated-by"),
    ("duplicated-by", "duplicates"),
];

/// Get the inverse of a link type
///
/// Returns the inverse link type for built-in types, or searches custom types.
/// Returns `None` if the link type is not found.
pub fn get_inverse_link_type(
    link_type: &str,
    custom_types: &[CustomLinkTypeDefinition],
) -> Option<String> {
    // Check built-in types first
    for (name, inverse) in BUILTIN_LINK_TYPES {
        if *name == link_type {
            return Some((*inverse).to_string());
        }
    }

    // Check custom types
    for custom in custom_types {
        if custom.name == link_type {
            return Some(custom.inverse.clone());
        }
        if custom.inverse == link_type {
            return Some(custom.name.clone());
        }
    }

    None
}

/// Check if a link type is valid (either built-in or custom)
pub fn is_valid_link_type(link_type: &str, custom_types: &[CustomLinkTypeDefinition]) -> bool {
    // Check built-in types
    for (name, _) in BUILTIN_LINK_TYPES {
        if *name == link_type {
            return true;
        }
    }

    // Check custom types
    for custom in custom_types {
        if custom.name == link_type || custom.inverse == link_type {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_target_type_from_str() {
        assert_eq!(TargetType::from_str("issue").ok(), Some(TargetType::Issue));
        assert_eq!(TargetType::from_str("doc").ok(), Some(TargetType::Doc));
        assert_eq!(TargetType::from_str("ISSUE").ok(), Some(TargetType::Issue));
        assert!(TargetType::from_str("pr").is_err());
        assert!(TargetType::from_str("unknown").is_err());
    }

    #[test]
    fn test_target_type_folder_name() {
        assert_eq!(TargetType::Issue.folder_name(), "issues");
        assert_eq!(TargetType::Doc.folder_name(), "docs");
    }

    #[test]
    fn test_get_inverse_builtin() {
        let custom: Vec<CustomLinkTypeDefinition> = vec![];
        assert_eq!(
            get_inverse_link_type("blocks", &custom),
            Some("blocked-by".to_string())
        );
        assert_eq!(
            get_inverse_link_type("blocked-by", &custom),
            Some("blocks".to_string())
        );
        assert_eq!(
            get_inverse_link_type("parent-of", &custom),
            Some("child-of".to_string())
        );
    }

    #[test]
    fn test_get_inverse_custom() {
        let custom = vec![CustomLinkTypeDefinition {
            name: "depends-on".to_string(),
            inverse: "dependency-of".to_string(),
            description: None,
        }];
        assert_eq!(
            get_inverse_link_type("depends-on", &custom),
            Some("dependency-of".to_string())
        );
        assert_eq!(
            get_inverse_link_type("dependency-of", &custom),
            Some("depends-on".to_string())
        );
    }

    #[test]
    fn test_get_inverse_unknown() {
        let custom: Vec<CustomLinkTypeDefinition> = vec![];
        assert_eq!(get_inverse_link_type("unknown-type", &custom), None);
    }

    #[test]
    fn test_is_valid_link_type() {
        let custom = vec![CustomLinkTypeDefinition {
            name: "depends-on".to_string(),
            inverse: "dependency-of".to_string(),
            description: None,
        }];
        assert!(is_valid_link_type("blocks", &custom));
        assert!(is_valid_link_type("depends-on", &custom));
        assert!(is_valid_link_type("dependency-of", &custom));
        assert!(!is_valid_link_type("invalid-type", &custom));
    }

    #[test]
    fn test_link_serialization() {
        let link = Link::new(
            "uuid-123".to_string(),
            TargetType::Issue,
            "blocks".to_string(),
        );
        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("\"targetId\":\"uuid-123\""));
        assert!(json.contains("\"targetType\":\"issue\""));
        assert!(json.contains("\"linkType\":\"blocks\""));
    }

    #[test]
    fn test_target_type_as_str() {
        assert_eq!(TargetType::Issue.as_str(), "issue");
        assert_eq!(TargetType::Doc.as_str(), "doc");
    }

    #[test]
    fn test_target_type_display() {
        assert_eq!(format!("{}", TargetType::Issue), "issue");
        assert_eq!(format!("{}", TargetType::Doc), "doc");
    }

    #[test]
    fn test_target_type_serialization() {
        let json = serde_json::to_string(&TargetType::Issue).unwrap();
        assert_eq!(json, "\"issue\"");

        let json = serde_json::to_string(&TargetType::Doc).unwrap();
        assert_eq!(json, "\"doc\"");
    }

    #[test]
    fn test_target_type_deserialization() {
        let tt: TargetType = serde_json::from_str("\"issue\"").unwrap();
        assert_eq!(tt, TargetType::Issue);

        let tt: TargetType = serde_json::from_str("\"doc\"").unwrap();
        assert_eq!(tt, TargetType::Doc);
    }

    #[test]
    fn test_target_type_eq() {
        assert_eq!(TargetType::Issue, TargetType::Issue);
        assert_ne!(TargetType::Issue, TargetType::Doc);
    }

    #[test]
    fn test_link_new_creates_timestamp() {
        let link = Link::new(
            "target-1".to_string(),
            TargetType::Doc,
            "relates-to".to_string(),
        );
        assert_eq!(link.target_id, "target-1");
        assert_eq!(link.target_type, TargetType::Doc);
        assert_eq!(link.link_type, "relates-to");
        assert!(!link.created_at.is_empty());
    }

    #[test]
    fn test_link_deserialization() {
        let json = r#"{
            "targetId": "abc-123",
            "targetType": "issue",
            "linkType": "blocks",
            "createdAt": "2024-01-01T00:00:00Z"
        }"#;

        let link: Link = serde_json::from_str(json).unwrap();
        assert_eq!(link.target_id, "abc-123");
        assert_eq!(link.target_type, TargetType::Issue);
        assert_eq!(link.link_type, "blocks");
        assert_eq!(link.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_custom_link_type_definition_serialization() {
        let def = CustomLinkTypeDefinition {
            name: "depends-on".to_string(),
            inverse: "dependency-of".to_string(),
            description: Some("Dependency relationship".to_string()),
        };

        let json = serde_json::to_string(&def).unwrap();
        let deserialized: CustomLinkTypeDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "depends-on");
        assert_eq!(deserialized.inverse, "dependency-of");
        assert_eq!(
            deserialized.description,
            Some("Dependency relationship".to_string())
        );
    }

    #[test]
    fn test_custom_link_type_definition_without_description() {
        let def = CustomLinkTypeDefinition {
            name: "test".to_string(),
            inverse: "test-inverse".to_string(),
            description: None,
        };

        let json = serde_json::to_string(&def).unwrap();
        assert!(!json.contains("description"));
    }

    #[test]
    fn test_builtin_link_types_count() {
        assert_eq!(BUILTIN_LINK_TYPES.len(), 8); // 4 pairs = 8 entries
    }

    #[test]
    fn test_builtin_link_types_symmetry() {
        // Every forward type should have its inverse present
        for (name, inverse) in BUILTIN_LINK_TYPES {
            let has_inverse = BUILTIN_LINK_TYPES.iter().any(|(n, _)| *n == *inverse);
            assert!(
                has_inverse,
                "Inverse '{inverse}' of '{name}' not found in BUILTIN_LINK_TYPES"
            );
        }
    }

    #[test]
    fn test_is_valid_link_type_all_builtins() {
        let custom: Vec<CustomLinkTypeDefinition> = vec![];
        for (name, _) in BUILTIN_LINK_TYPES {
            assert!(is_valid_link_type(name, &custom), "{name} should be valid");
        }
    }

    #[test]
    fn test_get_inverse_all_builtins() {
        let custom: Vec<CustomLinkTypeDefinition> = vec![];
        for (name, expected_inverse) in BUILTIN_LINK_TYPES {
            let inverse = get_inverse_link_type(name, &custom);
            assert_eq!(
                inverse.as_deref(),
                Some(*expected_inverse),
                "Inverse of '{name}' should be '{expected_inverse}'"
            );
        }
    }
}
