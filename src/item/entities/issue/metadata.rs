use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::CommonMetadata;

/// Frontmatter metadata for the new YAML-based issue format.
///
/// This struct is serialized to YAML frontmatter in `.centy/issues/{uuid}.md` files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssueFrontmatter {
    /// Human-readable display number (1, 2, 3...)
    pub display_number: u32,
    /// Issue status
    pub status: String,
    /// Priority as a number (1 = highest, N = lowest)
    pub priority: u32,
    /// ISO timestamp when the issue was created
    pub created_at: String,
    /// ISO timestamp when the issue was last updated
    pub updated_at: String,
    /// Whether this issue is a draft
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub draft: bool,
    /// ISO timestamp when soft-deleted (None if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    /// Whether this issue is an organization-level issue
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_org_issue: bool,
    /// Organization slug for org issues
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
    /// Org-scoped display number (consistent across all org projects)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_display_number: Option<u32>,
    /// Custom fields
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, String>,
}

impl IssueFrontmatter {
    /// Create new frontmatter from IssueMetadata and custom fields
    #[must_use]
    pub fn from_metadata(metadata: &IssueMetadata, custom_fields: HashMap<String, String>) -> Self {
        Self {
            display_number: metadata.common.display_number,
            status: metadata.common.status.clone(),
            priority: metadata.common.priority,
            created_at: metadata.common.created_at.clone(),
            updated_at: metadata.common.updated_at.clone(),
            draft: metadata.draft,
            deleted_at: metadata.deleted_at.clone(),
            is_org_issue: metadata.is_org_issue,
            org_slug: metadata.org_slug.clone(),
            org_display_number: metadata.org_display_number,
            custom_fields,
        }
    }

    /// Convert to IssueMetadata for internal use
    #[must_use]
    pub fn to_metadata(&self) -> IssueMetadata {
        IssueMetadata {
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
            draft: self.draft,
            deleted_at: self.deleted_at.clone(),
            is_org_issue: self.is_org_issue,
            org_slug: self.org_slug.clone(),
            org_display_number: self.org_display_number,
        }
    }
}

/// Legacy JSON metadata for backward compatibility.
/// This struct is used for reading from `metadata.json` files in the old format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueMetadata {
    /// Common fields shared with PRs (flattened for backward-compatible JSON)
    #[serde(flatten)]
    pub common: CommonMetadata,
    /// Whether this issue is a draft
    #[serde(default)]
    pub draft: bool,
    /// ISO timestamp when soft-deleted (None if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    /// Whether this issue is an organization-level issue (synced across org projects)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_org_issue: bool,
    /// Organization slug for org issues (for traceability)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
    /// Org-scoped display number (consistent across all org projects)
    /// Only set for org issues; project-local issues use common.display_number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_display_number: Option<u32>,
}

impl IssueMetadata {
    #[must_use]
    pub fn new(
        display_number: u32,
        status: String,
        priority: u32,
        custom_fields: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            common: CommonMetadata::new(display_number, status, priority, custom_fields),
            draft: false,
            deleted_at: None,
            is_org_issue: false,
            org_slug: None,
            org_display_number: None,
        }
    }

    #[must_use]
    pub fn new_draft(
        display_number: u32,
        status: String,
        priority: u32,
        custom_fields: HashMap<String, serde_json::Value>,
        draft: bool,
    ) -> Self {
        Self {
            common: CommonMetadata::new(display_number, status, priority, custom_fields),
            draft,
            deleted_at: None,
            is_org_issue: false,
            org_slug: None,
            org_display_number: None,
        }
    }

    /// Create metadata for an organization-level issue.
    ///
    /// Org issues are synced across all projects in an organization.
    /// They have both a local display number (per project) and an org-level
    /// display number (consistent across all org projects).
    #[must_use]
    pub fn new_org_issue(
        display_number: u32,
        org_display_number: u32,
        status: String,
        priority: u32,
        org_slug: &str,
        custom_fields: HashMap<String, serde_json::Value>,
        draft: bool,
    ) -> Self {
        Self {
            common: CommonMetadata::new(display_number, status, priority, custom_fields),
            draft,
            deleted_at: None,
            is_org_issue: true,
            org_slug: Some(org_slug.to_string()),
            org_display_number: Some(org_display_number),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_priority_number() {
        let json =
            r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.common.priority, 1);
    }

    #[test]
    fn test_deserialize_priority_string_high() {
        let json = r#"{"status":"open","priority":"high","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.common.priority, 1);
    }

    #[test]
    fn test_deserialize_priority_string_medium() {
        let json = r#"{"status":"open","priority":"medium","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.common.priority, 2);
    }

    #[test]
    fn test_deserialize_priority_string_low() {
        let json = r#"{"status":"open","priority":"low","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.common.priority, 3);
    }

    #[test]
    fn test_serialize_priority_as_number() {
        let metadata = IssueMetadata::new(1, "open".to_string(), 2, HashMap::new());
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""priority":2"#));
    }

    #[test]
    fn test_metadata_new() {
        let metadata = IssueMetadata::new(1, "open".to_string(), 1, HashMap::new());
        assert_eq!(metadata.common.display_number, 1);
        assert_eq!(metadata.common.status, "open");
        assert_eq!(metadata.common.priority, 1);
        assert!(!metadata.common.created_at.is_empty());
        assert!(!metadata.common.updated_at.is_empty());
    }

    #[test]
    fn test_deserialize_legacy_without_display_number() {
        // Legacy issues without display_number should default to 0
        let json =
            r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.common.display_number, 0);
    }

    #[test]
    fn test_serialize_display_number() {
        let metadata = IssueMetadata::new(42, "open".to_string(), 1, HashMap::new());
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""displayNumber":42"#));
    }

    #[test]
    fn test_flatten_produces_flat_json() {
        // Verify that #[serde(flatten)] produces flat JSON, not nested under "common"
        let metadata = IssueMetadata::new(1, "open".to_string(), 2, HashMap::new());
        let json = serde_json::to_string(&metadata).unwrap();
        // Should NOT contain "common" as a key
        assert!(!json.contains(r#""common""#));
        // Should contain flattened fields directly
        assert!(json.contains(r#""displayNumber""#));
        assert!(json.contains(r#""status""#));
        assert!(json.contains(r#""priority""#));
    }
}
