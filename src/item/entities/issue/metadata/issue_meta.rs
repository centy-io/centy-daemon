use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use mdstore::CommonMetadata;
/// Legacy JSON metadata for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueMetadata {
    #[serde(flatten)]
    pub common: CommonMetadata,
    #[serde(default)]
    pub draft: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_org_issue: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_display_number: Option<u32>,
}
impl IssueMetadata {
    #[must_use]
    pub fn new(display_number: u32, status: String, priority: u32,
               custom_fields: HashMap<String, serde_json::Value>) -> Self {
        Self {
            common: CommonMetadata::new(display_number, status, priority, custom_fields),
            draft: false, deleted_at: None, is_org_issue: false, org_slug: None, org_display_number: None,
        }
    }
    #[must_use]
    pub fn new_draft(display_number: u32, status: String, priority: u32,
                     custom_fields: HashMap<String, serde_json::Value>, draft: bool) -> Self {
        Self {
            common: CommonMetadata::new(display_number, status, priority, custom_fields),
            draft, deleted_at: None, is_org_issue: false, org_slug: None, org_display_number: None,
        }
    }
    #[must_use]
    pub fn new_org_issue(display_number: u32, org_display_number: u32, status: String,
                         priority: u32, org_slug: &str,
                         custom_fields: HashMap<String, serde_json::Value>, draft: bool) -> Self {
        Self {
            common: CommonMetadata::new(display_number, status, priority, custom_fields),
            draft, deleted_at: None, is_org_issue: true,
            org_slug: Some(org_slug.to_string()), org_display_number: Some(org_display_number),
        }
    }
}
