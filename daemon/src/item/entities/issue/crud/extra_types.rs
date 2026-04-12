use super::types::Issue;
use crate::manifest::CentyManifest;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct IssueWithProject {
    pub issue: Issue,
    pub project_path: String,
    pub project_name: String,
}

#[derive(Debug, Clone)]
pub struct GetIssuesByUuidResult {
    pub issues: Vec<IssueWithProject>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MoveIssueOptions {
    pub source_project_path: PathBuf,
    pub target_project_path: PathBuf,
    pub issue_id: String,
}

#[derive(Debug, Clone)]
pub struct MoveIssueResult {
    pub issue: Issue,
    pub old_display_number: u32,
    pub source_manifest: CentyManifest,
    pub target_manifest: CentyManifest,
}
