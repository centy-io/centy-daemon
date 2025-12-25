use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::CommonMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueMetadata {
    /// Common fields shared with PRs (flattened for backward-compatible JSON)
    #[serde(flatten)]
    pub common: CommonMetadata,
    /// Whether this issue has been compacted into features
    #[serde(default)]
    pub compacted: bool,
    /// ISO timestamp when the issue was compacted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compacted_at: Option<String>,
    /// Whether this issue is a draft
    #[serde(default)]
    pub draft: bool,
    /// ISO timestamp when soft-deleted (None if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
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
            compacted: false,
            compacted_at: None,
            draft: false,
            deleted_at: None,
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
            compacted: false,
            compacted_at: None,
            draft,
            deleted_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_priority_number() {
        let json = r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
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
        let json = r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
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
