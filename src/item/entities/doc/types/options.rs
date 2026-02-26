use super::Doc;
use crate::manifest::CentyManifest;
use std::path::PathBuf;
/// Options for creating a doc
#[derive(Debug, Clone, Default)]
pub struct CreateDocOptions {
    pub title: String,
    pub content: String,
    pub slug: Option<String>,
    pub template: Option<String>,
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
