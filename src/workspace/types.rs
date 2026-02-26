use crate::item::entities::issue::Issue;
use std::path::PathBuf;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("IO error: {0}")] IoError(#[from] std::io::Error),
    #[error("Git error: {0}")] GitError(String),
    #[error("Issue error: {0}")] IssueError(#[from] crate::item::entities::issue::IssueCrudError),
}
pub struct CreateWorkspaceOptions {
    pub source_project_path: PathBuf,
    pub issue: Issue,
}
pub struct CreateWorkspaceResult {
    pub workspace_path: PathBuf,
    pub workspace_reused: bool,
}
pub struct CreateStandaloneWorkspaceOptions {
    pub source_project_path: PathBuf,
    pub name: Option<String>,
}
pub struct CreateStandaloneWorkspaceResult {
    pub workspace_path: PathBuf,
    pub workspace_id: String,
    pub workspace_name: String,
    pub workspace_reused: bool,
}
