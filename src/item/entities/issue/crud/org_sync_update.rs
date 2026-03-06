use super::super::assets::copy_assets_folder;
use super::super::metadata::IssueFrontmatter;
use super::org_sync::create_issue_in_project;
use super::read::{read_issue_from_frontmatter, read_issue_from_legacy_folder};
use super::types::{Issue, IssueMetadataFlat};
use crate::common::OrgSyncError;
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{get_centy_path, now_iso, CENTY_HEADER_YAML};
use mdstore::generate_frontmatter;
use std::path::Path;
use tokio::fs;

async fn read_existing_issue(
    file_path: &Path,
    folder_path: &Path,
    issue_id: &str,
) -> Option<(bool, Issue)> {
    if file_path.exists() {
        read_issue_from_frontmatter(file_path, issue_id)
            .await
            .ok()
            .map(|i| (true, i))
    } else if folder_path.exists() {
        read_issue_from_legacy_folder(folder_path, issue_id)
            .await
            .ok()
            .map(|i| (false, i))
    } else {
        None
    }
}
fn build_update_frontmatter(
    existing: &Issue,
    source_metadata: &IssueMetadataFlat,
    org_slug: &str,
) -> IssueFrontmatter {
    IssueFrontmatter {
        display_number: existing.metadata.display_number,
        status: source_metadata.status.clone(),
        priority: source_metadata.priority,
        created_at: existing.metadata.created_at.clone(),
        updated_at: now_iso(),
        draft: source_metadata.draft,
        deleted_at: source_metadata.deleted_at.clone(),
        is_org_issue: true,
        org_slug: Some(org_slug.to_string()),
        org_display_number: source_metadata.org_display_number,
        custom_fields: source_metadata.custom_fields.clone(),
    }
}
pub async fn update_or_create_issue_in_project(
    project_path: &Path,
    issue_id: &str,
    title: &str,
    description: &str,
    source_metadata: &IssueMetadataFlat,
    org_slug: &str,
) -> Result<(), OrgSyncError> {
    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    let issue_file_path = issues_path.join(format!("{issue_id}.md"));
    let issue_folder_path = issues_path.join(issue_id);
    let Some((is_new_format, existing)) =
        read_existing_issue(&issue_file_path, &issue_folder_path, issue_id).await
    else {
        return create_issue_in_project(
            project_path,
            issue_id,
            title,
            description,
            source_metadata,
            org_slug,
        )
        .await;
    };
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?
        .ok_or_else(|| OrgSyncError::SyncFailed("Target project not initialized".to_string()))?;
    let frontmatter = build_update_frontmatter(&existing, source_metadata, org_slug);
    let issue_content =
        generate_frontmatter(&frontmatter, title, description, Some(CENTY_HEADER_YAML));
    fs::write(&issue_file_path, &issue_content).await?;
    if !is_new_format {
        let old_assets_path = issue_folder_path.join("assets");
        let new_assets_path = issues_path.join("assets").join(issue_id);
        if old_assets_path.exists() {
            fs::create_dir_all(&new_assets_path).await?;
            let _ = copy_assets_folder(&old_assets_path, &new_assets_path).await;
        }
        let _ = fs::remove_dir_all(&issue_folder_path).await;
    }
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?;
    Ok(())
}
