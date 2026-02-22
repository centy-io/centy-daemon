use crate::utils::now_iso;

/// Doc metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocMetadata {
    pub created_at: String,
    pub updated_at: String,
    /// ISO timestamp when soft-deleted (None if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    /// Whether this doc is organization-level (synced on creation)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_org_doc: bool,
    /// Organization slug for org docs (for traceability)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
}

impl DocMetadata {
    #[must_use]
    pub fn new() -> Self {
        let now = now_iso();
        Self {
            created_at: now.clone(),
            updated_at: now,
            deleted_at: None,
            is_org_doc: false,
            org_slug: None,
        }
    }

    #[must_use]
    pub fn new_org_doc(org_slug: &str) -> Self {
        let now = now_iso();
        Self {
            created_at: now.clone(),
            updated_at: now,
            deleted_at: None,
            is_org_doc: true,
            org_slug: Some(org_slug.to_string()),
        }
    }
}

impl Default for DocMetadata {
    fn default() -> Self {
        Self::new()
    }
}
