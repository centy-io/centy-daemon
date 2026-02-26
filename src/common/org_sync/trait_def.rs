use super::types::OrgSyncError;
use async_trait::async_trait;
use std::path::Path;
/// Trait for items that can be synced across organization projects.
#[async_trait]
pub trait OrgSyncable: Sized + Send + Sync {
    /// The organization slug, if this is an org item
    fn org_slug(&self) -> Option<&str>;
    /// Create or update this item in the target project.
    async fn sync_to_project(
        &self,
        target_project: &Path,
        org_slug: &str,
    ) -> Result<(), OrgSyncError>;
    /// Sync an update to the target project, with optional old ID for renames.
    async fn sync_update_to_project(
        &self,
        target_project: &Path,
        org_slug: &str,
        _old_id: Option<&str>,
    ) -> Result<(), OrgSyncError> {
        self.sync_to_project(target_project, org_slug).await
    }
}
