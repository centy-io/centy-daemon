use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use mdstore::CommonMetadata;
use super::IssueMetadata;
/// Frontmatter metadata for the new YAML-based issue format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssueFrontmatter {
    pub display_number: u32,
    pub status: String,
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub draft: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_org_issue: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_display_number: Option<u32>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, String>,
}
impl IssueFrontmatter {
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
    #[must_use]
    pub fn to_metadata(&self) -> IssueMetadata {
        IssueMetadata {
            common: CommonMetadata {
                display_number: self.display_number,
                status: self.status.clone(),
                priority: self.priority,
                created_at: self.created_at.clone(),
                updated_at: self.updated_at.clone(),
                custom_fields: self.custom_fields.iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone()))).collect(),
            },
            draft: self.draft,
            deleted_at: self.deleted_at.clone(),
            is_org_issue: self.is_org_issue,
            org_slug: self.org_slug.clone(),
            org_display_number: self.org_display_number,
        }
    }
}
