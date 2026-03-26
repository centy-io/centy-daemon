use super::types::TargetType;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum LinkError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Store error: {0}")]
    StoreError(#[from] mdstore::StoreError),
    #[error("Invalid link type: {0}")]
    InvalidLinkType(String),
    #[error("Source entity not found: {0} ({1})")]
    SourceNotFound(String, TargetType),
    #[error("Target entity not found: {0} ({1})")]
    TargetNotFound(String, TargetType),
    #[error("Link already exists")]
    LinkAlreadyExists,
    #[error("Link not found")]
    LinkNotFound,
    #[error("Cannot link entity to itself")]
    SelfLink,
}
/// Options for creating a link
#[derive(Debug, Clone)]
pub struct CreateLinkOptions {
    pub source_id: String,
    pub source_type: TargetType,
    pub target_id: String,
    pub target_type: TargetType,
    pub link_type: String,
}
/// Options for deleting a link (by UUID)
#[derive(Debug, Clone)]
pub struct DeleteLinkOptions {
    /// UUID of the link file to delete (from `Link.id` in the proto response).
    pub link_id: String,
}
/// Result of deleting a link
#[derive(Debug)]
pub struct DeleteLinkResult {
    pub deleted_count: u32,
}
/// Information about a link type
#[derive(Debug, Clone)]
pub struct LinkTypeInfo {
    pub name: String,
    pub description: Option<String>,
    pub is_builtin: bool,
}
