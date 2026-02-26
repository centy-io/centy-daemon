use super::super::metadata::IssueFrontmatter;
use super::super::reconcile::get_next_display_number;
use super::org_sync_update::update_or_create_issue_in_project;
use super::types::{Issue, IssueMetadataFlat};
use crate::common::{OrgSyncError, OrgSyncable};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{format_markdown, get_centy_path, now_iso};
use async_trait::async_trait;
use mdstore::generate_frontmatter;
use std::path::Path;
use tokio::fs;

#[async_trait]
impl OrgSyncable for Issue {
    fn org_slug(&self) -> Option<&str> {
        self.metadata.org_slug.as_deref()
    }

    async fn sync_to_project(
        &self,
        target_project: &Path,
        org_slug: &str,
    ) -> Result<(), OrgSyncError> {
        create_issue_in_project(
            target_project,
            &self.id,
            &self.title,
            &self.description,
            &self.metadata,
            org_slug,
        )
        .await
    }

    async fn sync_update_to_project(
        &self,
        target_project: &Path,
        org_slug: &str,
        _old_id: Option<&str>,
    ) -> Result<(), OrgSyncError> {
        update_or_create_issue_in_project(
            target_project,
            &self.id,
            &self.title,
            &self.description,
            &self.metadata,
            org_slug,
        )
        .await
    }
}

pub async fn create_issue_in_project(
    project_path: &Path,
    issue_id: &str,
    title: &str,
    description: &str,
    source_metadata: &IssueMetadataFlat,
    org_slug: &str,
) -> Result<(), OrgSyncError> {
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?
        .ok_or_else(|| OrgSyncError::SyncFailed("Target project not initialized".to_string()))?;
    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    let issue_file_path = issues_path.join(format!("{issue_id}.md"));
    let issue_folder_path = issues_path.join(issue_id);
    if issue_file_path.exists() || issue_folder_path.exists() {
        return Ok(());
    }
    fs::create_dir_all(&issues_path).await?;
    let local_display_number = get_next_display_number(&issues_path)
        .await
        .map_err(|e| OrgSyncError::SyncFailed(e.to_string()))?;
    let now = now_iso();
    let frontmatter = IssueFrontmatter {
        display_number: local_display_number,
        status: source_metadata.status.clone(),
        priority: source_metadata.priority,
        created_at: now.clone(),
        updated_at: now,
        draft: source_metadata.draft,
        deleted_at: None,
        is_org_issue: true,
        org_slug: Some(org_slug.to_string()),
        org_display_number: source_metadata.org_display_number,
        custom_fields: source_metadata.custom_fields.clone(),
    };
    let issue_content = generate_frontmatter(&frontmatter, title, description);
    fs::write(&issue_file_path, format_markdown(&issue_content)).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?;
    Ok(())
}
