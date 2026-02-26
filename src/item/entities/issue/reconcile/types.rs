use thiserror::Error;
#[derive(Error, Debug)]
pub enum ReconcileError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("YAML frontmatter error: {0}")]
    FrontmatterError(#[from] mdstore::FrontmatterError),
}
/// Information about an issue needed for reconciliation
#[derive(Debug, Clone)]
pub struct IssueInfo {
    /// Issue ID (UUID)
    pub id: String,
    /// Whether this is a new format (.md file) or old format (folder)
    pub is_new_format: bool,
    pub display_number: u32,
    pub created_at: String,
}
