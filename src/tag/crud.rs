//! Tag CRUD operations.

use super::storage::{find_tag_by_name, find_tag_index_by_name, read_tags, write_tags};
use super::types::{slugify_tag_name, validate_color, validate_tag_name, Tag, TagError};
use crate::common::org_sync::{sync_to_org_projects, sync_update_to_org_projects, OrgSyncError, OrgSyncResult, OrgSyncable};
use crate::manifest::{read_manifest, update_manifest_timestamp, write_manifest, CentyManifest};
use crate::utils::now_iso;
use async_trait::async_trait;
use std::path::Path;
use tracing::info;

/// Options for creating a tag
pub struct CreateTagOptions {
    pub name: String,
    pub color: Option<String>,
    pub is_org_tag: bool,
    pub org_slug: Option<String>,
}

/// Result of creating a tag
pub struct CreateTagResult {
    pub tag: Tag,
    pub manifest: CentyManifest,
    pub sync_results: Vec<OrgSyncResult>,
}

/// Create a new tag
pub async fn create_tag(
    project_path: &Path,
    options: CreateTagOptions,
) -> Result<CreateTagResult, TagError> {
    // Slugify and validate name
    let name = slugify_tag_name(&options.name);
    validate_tag_name(&name)?;

    // Validate color if provided
    if let Some(ref color) = options.color {
        validate_color(color)?;
    }

    // Read manifest
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| TagError::NotInitialized)?
        .ok_or(TagError::NotInitialized)?;

    // Read existing tags
    let mut tags = read_tags(project_path).await?;

    // Check if tag already exists
    if find_tag_by_name(&tags, &name).is_some() {
        return Err(TagError::TagAlreadyExists(name));
    }

    // Create new tag
    let now = now_iso();
    let tag = Tag {
        name: name.clone(),
        color: options.color,
        created_at: now,
        is_org_tag: options.is_org_tag,
        org_slug: options.org_slug,
    };

    tags.push(tag.clone());

    // Write tags (sorting is done in write_tags)
    write_tags(project_path, &tags).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    // Sync to org projects if this is an org tag
    let sync_results = if tag.is_org_tag {
        sync_to_org_projects(&tag, project_path).await
    } else {
        Vec::new()
    };

    info!("Created tag: {}", name);

    Ok(CreateTagResult {
        tag,
        manifest,
        sync_results,
    })
}

/// Get a tag by name
pub async fn get_tag(project_path: &Path, name: &str) -> Result<Tag, TagError> {
    let tags = read_tags(project_path).await?;

    tags.into_iter()
        .find(|t| t.name == name)
        .ok_or_else(|| TagError::TagNotFound(name.to_string()))
}

/// List all tags
pub async fn list_tags(project_path: &Path) -> Result<Vec<Tag>, TagError> {
    read_tags(project_path).await
}

/// Options for updating a tag
pub struct UpdateTagOptions {
    pub new_name: Option<String>,
    pub color: Option<String>,
}

/// Result of updating a tag
pub struct UpdateTagResult {
    pub tag: Tag,
    pub manifest: CentyManifest,
    pub sync_results: Vec<OrgSyncResult>,
}

/// Update an existing tag
pub async fn update_tag(
    project_path: &Path,
    tag_name: &str,
    options: UpdateTagOptions,
) -> Result<UpdateTagResult, TagError> {
    // Read manifest
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| TagError::NotInitialized)?
        .ok_or(TagError::NotInitialized)?;

    // Read existing tags
    let mut tags = read_tags(project_path).await?;

    // Find tag
    let tag_idx = find_tag_index_by_name(&tags, tag_name)
        .ok_or_else(|| TagError::TagNotFound(tag_name.to_string()))?;

    let old_name = tags[tag_idx].name.clone();

    // Update name if provided
    if let Some(ref new_name) = options.new_name {
        let slugified_name = slugify_tag_name(new_name);
        validate_tag_name(&slugified_name)?;

        // Check if new name already exists (if different from current)
        if slugified_name != old_name && find_tag_by_name(&tags, &slugified_name).is_some() {
            return Err(TagError::TagAlreadyExists(slugified_name));
        }

        tags[tag_idx].name = slugified_name;
    }

    // Update color if provided
    if let Some(ref color) = options.color {
        if color.is_empty() {
            tags[tag_idx].color = None;
        } else {
            validate_color(color)?;
            tags[tag_idx].color = Some(color.clone());
        }
    }

    let updated_tag = tags[tag_idx].clone();

    // Write tags
    write_tags(project_path, &tags).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    // Sync to org projects if this is an org tag
    let sync_results = if updated_tag.is_org_tag {
        let old_id = if old_name == updated_tag.name {
            None
        } else {
            Some(old_name.as_str())
        };
        sync_update_to_org_projects(&updated_tag, project_path, old_id).await
    } else {
        Vec::new()
    };

    info!("Updated tag: {}", updated_tag.name);

    Ok(UpdateTagResult {
        tag: updated_tag,
        manifest,
        sync_results,
    })
}

/// Result of deleting a tag
pub struct DeleteTagResult {
    pub manifest: CentyManifest,
}

/// Delete a tag
pub async fn delete_tag(project_path: &Path, tag_name: &str) -> Result<DeleteTagResult, TagError> {
    // Read manifest
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| TagError::NotInitialized)?
        .ok_or(TagError::NotInitialized)?;

    // Read existing tags
    let mut tags = read_tags(project_path).await?;

    // Find tag index
    let tag_idx = find_tag_index_by_name(&tags, tag_name)
        .ok_or_else(|| TagError::TagNotFound(tag_name.to_string()))?;

    // Remove tag
    tags.remove(tag_idx);

    // Write tags
    write_tags(project_path, &tags).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    info!("Deleted tag: {}", tag_name);

    Ok(DeleteTagResult { manifest })
}

// ============ OrgSyncable Implementation ============

#[async_trait]
impl OrgSyncable for Tag {
    fn item_id(&self) -> &str {
        &self.name
    }

    fn is_org_item(&self) -> bool {
        self.is_org_tag
    }

    fn org_slug(&self) -> Option<&str> {
        self.org_slug.as_deref()
    }

    async fn sync_to_project(
        &self,
        target_project: &Path,
        _org_slug: &str,
    ) -> Result<(), OrgSyncError> {
        // Read existing tags in target project
        let mut tags = read_tags(target_project)
            .await
            .map_err(|e| OrgSyncError::SyncFailed(e.to_string()))?;

        // Check if tag already exists
        if let Some(idx) = find_tag_index_by_name(&tags, &self.name) {
            // Update existing tag
            tags[idx] = self.clone();
        } else {
            // Add new tag
            tags.push(self.clone());
        }

        // Write tags
        write_tags(target_project, &tags)
            .await
            .map_err(|e| OrgSyncError::SyncFailed(e.to_string()))?;

        // Update manifest
        let mut manifest = read_manifest(target_project)
            .await
            .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?
            .ok_or_else(|| OrgSyncError::ManifestError("No manifest found".to_string()))?;

        update_manifest_timestamp(&mut manifest);
        write_manifest(target_project, &manifest)
            .await
            .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?;

        Ok(())
    }

    async fn sync_update_to_project(
        &self,
        target_project: &Path,
        org_slug: &str,
        old_id: Option<&str>,
    ) -> Result<(), OrgSyncError> {
        // Read existing tags in target project
        let mut tags = read_tags(target_project)
            .await
            .map_err(|e| OrgSyncError::SyncFailed(e.to_string()))?;

        // If old_id is provided (rename case), remove the old tag first
        if let Some(old_name) = old_id {
            if let Some(idx) = find_tag_index_by_name(&tags, old_name) {
                tags.remove(idx);
            }
        }

        // Now sync the updated tag
        if let Some(idx) = find_tag_index_by_name(&tags, &self.name) {
            // Update existing tag
            tags[idx] = self.clone();
        } else {
            // Add new tag
            tags.push(self.clone());
        }

        // Write tags
        write_tags(target_project, &tags)
            .await
            .map_err(|e| OrgSyncError::SyncFailed(e.to_string()))?;

        // Update manifest
        let mut manifest = read_manifest(target_project)
            .await
            .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?
            .ok_or_else(|| OrgSyncError::ManifestError("No manifest found".to_string()))?;

        update_manifest_timestamp(&mut manifest);
        write_manifest(target_project, &manifest)
            .await
            .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?;

        // Suppress unused variable warning
        let _ = org_slug;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tag_options() {
        let options = CreateTagOptions {
            name: "Bug Fix".to_string(),
            color: Some("#ef4444".to_string()),
            is_org_tag: false,
            org_slug: None,
        };

        assert_eq!(slugify_tag_name(&options.name), "bug-fix");
    }

    #[test]
    fn test_update_tag_options() {
        let options = UpdateTagOptions {
            new_name: Some("Feature Request".to_string()),
            color: Some("#10b981".to_string()),
        };

        assert!(options.new_name.is_some());
        assert!(options.color.is_some());
    }
}
