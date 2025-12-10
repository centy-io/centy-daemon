use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use crate::issue::priority::migrate_string_priority;

/// Default priority levels for migration when config is not available
const DEFAULT_PRIORITY_LEVELS: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrMetadata {
    /// Human-readable display number (1, 2, 3...).
    /// Used for user-facing references while folder uses UUID.
    #[serde(default)]
    pub display_number: u32,
    pub status: String,
    pub source_branch: String,
    pub target_branch: String,
    #[serde(default)]
    pub linked_issues: Vec<String>,
    #[serde(default)]
    pub reviewers: Vec<String>,
    /// Priority as a number (1 = highest, N = lowest).
    /// During deserialization, string values are automatically migrated to numbers.
    #[serde(deserialize_with = "deserialize_priority")]
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    /// Timestamp when PR was merged (empty string if not merged)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub merged_at: String,
    /// Timestamp when PR was closed (empty string if not closed)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub closed_at: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl PrMetadata {
    #[must_use] 
    pub fn new(
        display_number: u32,
        status: String,
        source_branch: String,
        target_branch: String,
        linked_issues: Vec<String>,
        reviewers: Vec<String>,
        priority: u32,
        custom_fields: HashMap<String, serde_json::Value>,
    ) -> Self {
        let now = crate::utils::now_iso();
        Self {
            display_number,
            status,
            source_branch,
            target_branch,
            linked_issues,
            reviewers,
            priority,
            created_at: now.clone(),
            updated_at: now,
            merged_at: String::new(),
            closed_at: String::new(),
            custom_fields,
        }
    }
}

/// Custom deserializer that handles both string and number formats for priority.
/// This enables backward compatibility with existing PRs that use string priorities.
fn deserialize_priority<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct PriorityVisitor;

    impl Visitor<'_> for PriorityVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a priority number or string")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as u32)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value < 0 {
                Err(E::custom("priority cannot be negative"))
            } else {
                Ok(value as u32)
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Migrate legacy string priority to number
            // Use default priority levels (3) for migration since we don't have config here
            Ok(migrate_string_priority(value, DEFAULT_PRIORITY_LEVELS))
        }
    }

    deserializer.deserialize_any(PriorityVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_priority_number() {
        let json = r#"{"status":"draft","sourceBranch":"feature","targetBranch":"main","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: PrMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 1);
    }

    #[test]
    fn test_deserialize_priority_string_high() {
        let json = r#"{"status":"draft","sourceBranch":"feature","targetBranch":"main","priority":"high","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: PrMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 1);
    }

    #[test]
    fn test_serialize_priority_as_number() {
        let metadata = PrMetadata::new(
            1,
            "draft".to_string(),
            "feature".to_string(),
            "main".to_string(),
            vec![],
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
            vec!["1".to_string(), "2".to_string()],
            vec!["alice".to_string()],
            1,
            HashMap::new(),
        );
        assert_eq!(metadata.display_number, 1);
        assert_eq!(metadata.status, "draft");
        assert_eq!(metadata.source_branch, "feature-branch");
        assert_eq!(metadata.target_branch, "main");
        assert_eq!(metadata.linked_issues.len(), 2);
        assert_eq!(metadata.reviewers.len(), 1);
        assert_eq!(metadata.priority, 1);
        assert!(!metadata.created_at.is_empty());
        assert!(!metadata.updated_at.is_empty());
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
            vec![],
            1,
            HashMap::new(),
        );
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""displayNumber":42"#));
    }
}
