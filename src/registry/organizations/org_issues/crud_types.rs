use super::paths::PathError;
use crate::item::entities::issue::org_registry::OrgIssueRegistryError;
use mdstore::FrontmatterError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum OrgIssueError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("YAML frontmatter error: {0}")]
    FrontmatterError(#[from] FrontmatterError),
    #[error("Path error: {0}")]
    PathError(#[from] PathError),
    #[error("Org registry error: {0}")]
    OrgRegistryError(#[from] OrgIssueRegistryError),
    #[error("Org issue not found: {0}")]
    NotFound(String),
    #[error("Title is required")]
    TitleRequired,
}
/// Frontmatter for org issue markdown files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrgIssueFrontmatter {
    pub display_number: u32,
    pub status: String,
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub referenced_projects: Vec<String>,
}
/// A fully loaded org issue
#[derive(Debug, Clone)]
pub struct OrgIssue {
    pub id: String,
    pub display_number: u32,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    pub custom_fields: HashMap<String, String>,
    pub referenced_projects: Vec<String>,
}
impl OrgIssue {}
/// Options for listing org issues
#[derive(Debug, Clone, Default)]
pub struct ListOrgIssuesOptions {
    pub status: Option<String>,
    pub priority: Option<u32>,
    pub referenced_project: Option<String>,
}
