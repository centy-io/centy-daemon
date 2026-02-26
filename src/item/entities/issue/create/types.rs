use super::super::org_registry::OrgIssueRegistryError;
use super::super::priority::PriorityError;
use super::super::reconcile::ReconcileError;
use super::super::status::StatusError;
use crate::common::OrgSyncResult;
use crate::manifest::CentyManifest;
use crate::template::TemplateError;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IssueError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Title is required")]
    TitleRequired,

    #[error("Invalid priority: {0}")]
    InvalidPriority(#[from] PriorityError),

    #[error("Invalid status: {0}")]
    InvalidStatus(#[from] StatusError),

    #[error("Template error: {0}")]
    TemplateError(#[from] TemplateError),

    #[error("Reconcile error: {0}")]
    ReconcileError(#[from] ReconcileError),

    #[error("Cannot create org issue: project has no organization")]
    NoOrganization,

    #[error("Org registry error: {0}")]
    OrgRegistryError(#[from] OrgIssueRegistryError),

    #[error("Registry error: {0}")]
    RegistryError(String),
}

/// Options for creating an issue
#[derive(Debug, Clone, Default)]
pub struct CreateIssueOptions {
    pub title: String,
    pub description: String,
    /// Priority as a number (1 = highest). None = use default.
    pub priority: Option<u32>,
    pub status: Option<String>,
    pub custom_fields: HashMap<String, String>,
    /// Optional template name (without .md extension)
    pub template: Option<String>,
    /// Whether to create the issue as a draft
    pub draft: Option<bool>,
    /// Create as organization-wide issue (syncs to all org projects)
    pub is_org_issue: bool,
}

/// Result of issue creation
#[derive(Debug, Clone)]
pub struct CreateIssueResult {
    /// UUID-based issue ID (folder name)
    pub id: String,
    /// Human-readable display number (1, 2, 3...)
    pub display_number: u32,
    /// Org-level display number (only for org issues)
    pub org_display_number: Option<u32>,
    /// Legacy field for backward compatibility (same as id)
    #[deprecated(note = "Use `id` instead")]
    pub issue_number: String,
    pub created_files: Vec<String>,
    pub manifest: CentyManifest,
    /// Results from syncing to other org projects (empty for non-org issues)
    pub sync_results: Vec<OrgSyncResult>,
}
