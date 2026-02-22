use crate::manifest::CentyManifest;
use crate::utils::now_iso;
use std::path::PathBuf;

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

/// Options for creating a doc
#[derive(Debug, Clone, Default)]
pub struct CreateDocOptions {
    pub title: String,
    pub content: String,
    pub slug: Option<String>,
    /// Optional template name (without .md extension)
    pub template: Option<String>,
    /// Create as organization-wide doc (syncs to all org projects)
    pub is_org_doc: bool,
}

/// Result of syncing an org doc to another project
#[derive(Debug, Clone)]
pub struct OrgDocSyncResult {
    pub project_path: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Result of doc creation
#[derive(Debug, Clone)]
pub struct CreateDocResult {
    pub slug: String,
    pub created_file: String,
    pub manifest: CentyManifest,
    /// Results from syncing to other org projects (empty for non-org docs)
    pub sync_results: Vec<OrgDocSyncResult>,
}

/// Options for updating a doc
#[derive(Debug, Clone, Default)]
pub struct UpdateDocOptions {
    pub title: Option<String>,
    pub content: Option<String>,
    pub new_slug: Option<String>,
}

/// Result of doc update
#[derive(Debug, Clone)]
pub struct UpdateDocResult {
    pub doc: Doc,
    pub manifest: CentyManifest,
    /// Results from syncing to other org projects (empty for non-org docs)
    pub sync_results: Vec<OrgDocSyncResult>,
}

/// A doc with its source project information
#[derive(Debug, Clone)]
pub struct DocWithProject {
    pub doc: Doc,
    pub project_path: String,
    pub project_name: String,
}

/// Result of searching for docs by slug across projects
#[derive(Debug, Clone)]
pub struct GetDocsBySlugResult {
    pub docs: Vec<DocWithProject>,
    pub errors: Vec<String>,
}

/// Options for moving a doc to another project
#[derive(Debug, Clone)]
pub struct MoveDocOptions {
    pub source_project_path: PathBuf,
    pub target_project_path: PathBuf,
    pub slug: String,
    pub new_slug: Option<String>,
}

/// Result of moving a doc
#[derive(Debug, Clone)]
pub struct MoveDocResult {
    pub doc: Doc,
    pub old_slug: String,
    pub source_manifest: CentyManifest,
    pub target_manifest: CentyManifest,
}
