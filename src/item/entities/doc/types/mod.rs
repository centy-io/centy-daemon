mod options;
pub use options::{
    CreateDocOptions, CreateDocResult, DocWithProject, GetDocsBySlugResult,
    MoveDocOptions, MoveDocResult, OrgDocSyncResult, UpdateDocOptions, UpdateDocResult,
};
use crate::utils::now_iso;
/// Full doc data
#[derive(Debug, Clone)]
pub struct Doc {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub metadata: DocMetadata,
}
/// Doc metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocMetadata {
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_org_doc: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
}
impl DocMetadata {
    #[must_use]
    pub fn new() -> Self {
        let now = now_iso();
        Self { created_at: now.clone(), updated_at: now, deleted_at: None, is_org_doc: false, org_slug: None }
    }
    #[must_use]
    pub fn new_org_doc(org_slug: &str) -> Self {
        let now = now_iso();
        Self { created_at: now.clone(), updated_at: now, deleted_at: None, is_org_doc: true, org_slug: Some(org_slug.to_string()) }
    }
}
impl Default for DocMetadata { fn default() -> Self { Self::new() } }
