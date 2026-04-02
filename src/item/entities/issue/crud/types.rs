use super::super::priority::PriorityError;
use super::super::reconcile::ReconcileError;
use super::super::status::StatusError;
use crate::manifest::CentyManifest;
use mdstore::FrontmatterError;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IssueCrudError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("YAML frontmatter error: {0}")]
    FrontmatterError(#[from] FrontmatterError),
    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,
    #[error("Issue {0} not found")]
    IssueNotFound(String),
    #[error("Issue with display number {0} not found")]
    IssueDisplayNumberNotFound(u32),
    #[error("Issue {0} is not soft-deleted")]
    IssueNotDeleted(String),
    #[error("Issue {0} is already soft-deleted")]
    IssueAlreadyDeleted(String),
    #[error("Invalid issue format: {0}")]
    InvalidIssueFormat(String),
    #[error("Invalid priority: {0}")]
    InvalidPriority(#[from] PriorityError),
    #[error("Invalid status: {0}")]
    InvalidStatus(#[from] StatusError),
    #[error("Reconcile error: {0}")]
    ReconcileError(#[from] ReconcileError),
    #[error("Target project not initialized")]
    TargetNotInitialized,
    #[error("Priority {0} exceeds target project's priority_levels")]
    InvalidPriorityInTarget(u32),
    #[error("Cannot move issue to same project")]
    SameProject,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub metadata: IssueMetadataFlat,
}

#[derive(Debug, Clone)]
pub struct IssueMetadataFlat {
    pub display_number: u32,
    pub status: String,
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    pub custom_fields: HashMap<String, String>,
    pub draft: bool,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateIssueOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<u32>,
    pub custom_fields: HashMap<String, String>,
    pub draft: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct UpdateIssueResult {
    pub issue: Issue,
    pub manifest: CentyManifest,
}
