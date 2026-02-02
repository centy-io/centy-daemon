use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::CommonMetadata;

/// Frontmatter metadata for the new YAML-based PR format.
///
/// This struct is serialized to YAML frontmatter in `.centy/prs/{uuid}.md` files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrFrontmatter {
    /// Human-readable display number (1, 2, 3...)
    pub display_number: u32,
    /// PR status
    pub status: String,
    /// Source branch name
    pub source_branch: String,
    /// Target branch name
    pub target_branch: String,
    /// Priority as a number (1 = highest, N = lowest)
    pub priority: u32,
    /// ISO timestamp when the PR was created
    pub created_at: String,
    /// ISO timestamp when the PR was last updated
    pub updated_at: String,
    /// Reviewers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviewers: Vec<String>,
    /// Timestamp when PR was merged (None if not merged)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<String>,
    /// Timestamp when PR was closed (None if not closed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    /// ISO timestamp when soft-deleted (None if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    /// Custom fields
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, String>,
}

impl PrFrontmatter {
    /// Convert to PrMetadata for internal use
    #[must_use]
    pub fn to_metadata(&self) -> PrMetadata {
        PrMetadata {
            common: CommonMetadata {
                display_number: self.display_number,
                status: self.status.clone(),
                priority: self.priority,
                created_at: self.created_at.clone(),
                updated_at: self.updated_at.clone(),
                custom_fields: self
                    .custom_fields
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect(),
            },
            source_branch: self.source_branch.clone(),
            target_branch: self.target_branch.clone(),
            reviewers: self.reviewers.clone(),
            merged_at: self.merged_at.clone().unwrap_or_default(),
            closed_at: self.closed_at.clone().unwrap_or_default(),
            deleted_at: self.deleted_at.clone(),
        }
    }
}

/// Legacy JSON metadata for backward compatibility.
/// This struct is used for reading from `metadata.json` files in the old format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrMetadata {
    /// Common fields shared with Issues (flattened for backward-compatible JSON)
    #[serde(flatten)]
    pub common: CommonMetadata,
    pub source_branch: String,
    pub target_branch: String,
    #[serde(default)]
    pub reviewers: Vec<String>,
    /// Timestamp when PR was merged (empty string if not merged)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub merged_at: String,
    /// Timestamp when PR was closed (empty string if not closed)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub closed_at: String,
    /// ISO timestamp when soft-deleted (None if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

impl PrMetadata {
    #[must_use]
    pub fn new(
        display_number: u32,
        status: String,
        source_branch: String,
        target_branch: String,
        reviewers: Vec<String>,
        priority: u32,
        custom_fields: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            common: CommonMetadata::new(display_number, status, priority, custom_fields),
            source_branch,
            target_branch,
            reviewers,
            merged_at: String::new(),
            closed_at: String::new(),
            deleted_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_priority_number() {
        let json = r#"{"status":"draft","sourceBranch":"feature","targetBranch":"main","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: PrMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.common.priority, 1);
    }

    #[test]
    fn test_deserialize_priority_string_high() {
        let json = r#"{"status":"draft","sourceBranch":"feature","targetBranch":"main","priority":"high","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: PrMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.common.priority, 1);
    }

    #[test]
    fn test_serialize_priority_as_number() {
        let metadata = PrMetadata::new(
            1,
            "draft".to_string(),
            "feature".to_string(),
            "main".to_string(),
            vec![],
            2,
            HashMap::new(),
        );
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""priority":2"#));
    }

    #[test]
    fn test_metadata_new() {
        let metadata = PrMetadata::new(
            1,
            "draft".to_string(),
            "feature-branch".to_string(),
            "main".to_string(),
            vec!["alice".to_string()],
            1,
            HashMap::new(),
        );
        assert_eq!(metadata.common.display_number, 1);
        assert_eq!(metadata.common.status, "draft");
        assert_eq!(metadata.source_branch, "feature-branch");
        assert_eq!(metadata.target_branch, "main");
        assert_eq!(metadata.reviewers.len(), 1);
        assert_eq!(metadata.common.priority, 1);
        assert!(!metadata.common.created_at.is_empty());
        assert!(!metadata.common.updated_at.is_empty());
        assert!(metadata.merged_at.is_empty());
        assert!(metadata.closed_at.is_empty());
    }

    #[test]
    fn test_serialize_display_number() {
        let metadata = PrMetadata::new(
            42,
            "open".to_string(),
            "feature".to_string(),
            "main".to_string(),
            vec![],
            1,
            HashMap::new(),
        );
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""displayNumber":42"#));
    }

    #[test]
    fn test_flatten_produces_flat_json() {
        // Verify that #[serde(flatten)] produces flat JSON, not nested under "common"
        let metadata = PrMetadata::new(
            1,
            "open".to_string(),
            "feature".to_string(),
            "main".to_string(),
            vec![],
            2,
            HashMap::new(),
        );
        let json = serde_json::to_string(&metadata).unwrap();
        // Should NOT contain "common" as a key
        assert!(!json.contains(r#""common""#));
        // Should contain flattened fields directly
        assert!(json.contains(r#""displayNumber""#));
        assert!(json.contains(r#""status""#));
        assert!(json.contains(r#""priority""#));
    }
}
