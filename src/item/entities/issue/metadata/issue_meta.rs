use mdstore::CommonMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        }
    }
}
