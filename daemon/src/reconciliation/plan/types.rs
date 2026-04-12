use crate::manifest::ManagedFileType;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum PlanError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
/// Information about a file in the reconciliation plan
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub file_type: ManagedFileType,
    pub hash: String,
    pub content_preview: Option<String>,
}
/// The reconciliation plan
#[derive(Debug, Clone, Default)]
pub struct ReconciliationPlan {
    /// Files that need to be created (not on disk, not in manifest)
    pub to_create: Vec<FileInfo>,
    /// Files that were deleted but exist in manifest (can be restored)
    pub to_restore: Vec<FileInfo>,
    /// Files that were modified from original (hash mismatch)
    pub to_reset: Vec<FileInfo>,
    /// Files that are up to date
    pub up_to_date: Vec<FileInfo>,
    /// User-created files (not managed by centy)
    pub user_files: Vec<FileInfo>,
}
impl ReconciliationPlan {
    /// Check if user decisions are needed
    #[must_use]
    pub fn needs_decisions(&self) -> bool {
        !self.to_restore.is_empty() || !self.to_reset.is_empty()
    }
}
