use super::assets::copy_assets_folder;
use super::id::{generate_issue_id, is_uuid, is_valid_issue_folder};
use super::metadata::IssueMetadata;
use super::planning::{
    add_planning_note, has_planning_note, is_planning_status, remove_planning_note,
};
use super::priority::{validate_priority, PriorityError};
use super::reconcile::{get_next_display_number, reconcile_display_numbers, ReconcileError};
use super::status::{validate_status, StatusError};
use crate::common::{OrgSyncError, OrgSyncable};
use crate::config::read_config;
use crate::manifest::{read_manifest, update_manifest_timestamp, write_manifest, CentyManifest};
use crate::registry::ProjectInfo;
use crate::utils::{format_markdown, get_centy_path, now_iso};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum IssueCrudError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Issue {0} not found")]
    IssueNotFound(String),

    #[error("Issue with display number {0} not found")]
    IssueDisplayNumberNotFound(u32),

    #[error("Issue {0} is not soft-deleted")]
    IssueNotDeleted(String),

    #[error("Issue {0} is already soft-deleted")]
    IssueAlreadyDeleted(String),

    #[error("Invalid issue format: {0}")]
    InvalidIssueFormat(String),

    #[error("Invalid priority: {0}")]
    InvalidPriority(#[from] PriorityError),

    #[error("Invalid status: {0}")]
    InvalidStatus(#[from] StatusError),

    #[error("Reconcile error: {0}")]
    ReconcileError(#[from] ReconcileError),

    #[error("Target project not initialized")]
    TargetNotInitialized,

    #[error("Priority {0} exceeds target project's priority_levels")]
    InvalidPriorityInTarget(u32),

    #[error("Cannot move issue to same project")]
    SameProject,
}

/// Full issue data
#[derive(Debug, Clone)]
pub struct Issue {
    /// UUID-based issue ID (folder name)
    pub id: String,
    /// Legacy field for backward compatibility (same as id)
    #[deprecated(note = "Use `id` instead")]
    pub issue_number: String,
    pub title: String,
    pub description: String,
    pub metadata: IssueMetadataFlat,
}

/// Flattened metadata for API responses
#[derive(Debug, Clone)]
pub struct IssueMetadataFlat {
    /// Human-readable display number (1, 2, 3...)
    pub display_number: u32,
    pub status: String,
    /// Priority as a number (1 = highest, N = lowest)
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    pub custom_fields: HashMap<String, String>,
    /// Whether this issue has been compacted into features
    pub compacted: bool,
    /// ISO timestamp when the issue was compacted
    pub compacted_at: Option<String>,
    /// Whether this issue is a draft
    pub draft: bool,
    /// ISO timestamp when soft-deleted (None if not deleted)
    pub deleted_at: Option<String>,
    /// Whether this issue is an organization-level issue
    pub is_org_issue: bool,
    /// Organization slug for org issues
    pub org_slug: Option<String>,
    /// Org-scoped display number (consistent across all org projects)
    pub org_display_number: Option<u32>,
}

/// Options for updating an issue
#[derive(Debug, Clone, Default)]
pub struct UpdateIssueOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    /// Priority as a number (1 = highest). None = don't update.
    pub priority: Option<u32>,
    pub custom_fields: HashMap<String, String>,
    /// Whether to mark as draft. None = don't update.
    pub draft: Option<bool>,
}

/// Result of issue update
#[derive(Debug, Clone)]
pub struct UpdateIssueResult {
    pub issue: Issue,
    pub manifest: CentyManifest,
    /// Results from syncing to other org projects (empty for non-org issues)
    pub sync_results: Vec<crate::common::OrgSyncResult>,
}

/// Result of issue deletion
#[derive(Debug, Clone)]
pub struct DeleteIssueResult {
    pub manifest: CentyManifest,
}

/// Result of soft-deleting an issue
#[derive(Debug, Clone)]
pub struct SoftDeleteIssueResult {
    pub issue: Issue,
    pub manifest: CentyManifest,
}

/// Result of restoring a soft-deleted issue
#[derive(Debug, Clone)]
pub struct RestoreIssueResult {
    pub issue: Issue,
    pub manifest: CentyManifest,
}

/// An issue with its source project information
#[derive(Debug, Clone)]
pub struct IssueWithProject {
    pub issue: Issue,
    pub project_path: String,
    pub project_name: String,
}

/// Result of searching for issues by UUID across projects
#[derive(Debug, Clone)]
pub struct GetIssuesByUuidResult {
    pub issues: Vec<IssueWithProject>,
    pub errors: Vec<String>,
}

/// Options for moving an issue to another project
#[derive(Debug, Clone)]
pub struct MoveIssueOptions {
    pub source_project_path: PathBuf,
    pub target_project_path: PathBuf,
    pub issue_id: String,
}

/// Result of moving an issue
#[derive(Debug, Clone)]
pub struct MoveIssueResult {
    pub issue: Issue,
    pub old_display_number: u32,
    pub source_manifest: CentyManifest,
    pub target_manifest: CentyManifest,
}

/// Options for duplicating an issue
#[derive(Debug, Clone)]
pub struct DuplicateIssueOptions {
    pub source_project_path: PathBuf,
    pub target_project_path: PathBuf,
    pub issue_id: String,
    pub new_title: Option<String>,
}

/// Result of duplicating an issue
#[derive(Debug, Clone)]
pub struct DuplicateIssueResult {
    pub issue: Issue,
    pub original_issue_id: String,
    pub manifest: CentyManifest,
}

/// Get a single issue by its number
pub async fn get_issue(project_path: &Path, issue_number: &str) -> Result<Issue, IssueCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issue_path = centy_path.join("issues").join(issue_number);

    if !issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    read_issue_from_disk(&issue_path, issue_number).await
}

/// List all issues with optional filtering
pub async fn list_issues(
    project_path: &Path,
    status_filter: Option<&str>,
    priority_filter: Option<u32>,
    draft_filter: Option<bool>,
    include_deleted: bool,
) -> Result<Vec<Issue>, IssueCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");

    if !issues_path.exists() {
        return Ok(Vec::new());
    }

    // Reconcile display numbers to resolve any conflicts from concurrent creation
    reconcile_display_numbers(&issues_path).await?;

    let mut issues = Vec::new();
    let mut entries = fs::read_dir(&issues_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(folder_name) = entry.file_name().to_str() {
                // Accept both UUID and legacy 4-digit format
                if !is_valid_issue_folder(folder_name) {
                    continue;
                }

                match read_issue_from_disk(&entry.path(), folder_name).await {
                    Ok(issue) => {
                        // Apply filters
                        let status_match = status_filter.is_none_or(|s| issue.metadata.status == s);
                        let priority_match =
                            priority_filter.is_none_or(|p| issue.metadata.priority == p);
                        let draft_match = draft_filter.is_none_or(|d| issue.metadata.draft == d);
                        let deleted_match = include_deleted || issue.metadata.deleted_at.is_none();

                        if status_match && priority_match && draft_match && deleted_match {
                            issues.push(issue);
                        }
                    }
                    Err(_) => {
                        // Skip issues that can't be read
                    }
                }
            }
        }
    }

    // Sort by display number (human-readable ordering)
    issues.sort_by_key(|i| i.metadata.display_number);

    Ok(issues)
}

/// Get an issue by its display number (human-readable number like 1, 2, 3)
pub async fn get_issue_by_display_number(
    project_path: &Path,
    display_number: u32,
) -> Result<Issue, IssueCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");

    if !issues_path.exists() {
        return Err(IssueCrudError::IssueDisplayNumberNotFound(display_number));
    }

    // Reconcile first to ensure display numbers are unique
    reconcile_display_numbers(&issues_path).await?;

    let mut entries = fs::read_dir(&issues_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(folder_name) = entry.file_name().to_str() {
                if !is_valid_issue_folder(folder_name) {
                    continue;
                }

                let metadata_path = entry.path().join("metadata.json");
                if !metadata_path.exists() {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&metadata_path).await {
                    if let Ok(metadata) = serde_json::from_str::<IssueMetadata>(&content) {
                        if metadata.common.display_number == display_number {
                            return read_issue_from_disk(&entry.path(), folder_name).await;
                        }
                    }
                }
            }
        }
    }

    Err(IssueCrudError::IssueDisplayNumberNotFound(display_number))
}

/// Struct holding applied update values after merging options with current issue
struct AppliedIssueUpdates {
    title: String,
    description: String,
    status: String,
    priority: u32,
    custom_fields: HashMap<String, String>,
    draft: bool,
}

/// Build updated metadata struct from current issue and applied updates
fn build_updated_metadata(current: &Issue, updates: &AppliedIssueUpdates) -> IssueMetadata {
    IssueMetadata {
        common: crate::common::CommonMetadata {
            display_number: current.metadata.display_number,
            status: updates.status.clone(),
            priority: updates.priority,
            created_at: current.metadata.created_at.clone(),
            updated_at: now_iso(),
            custom_fields: updates
                .custom_fields
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        },
        compacted: current.metadata.compacted,
        compacted_at: current.metadata.compacted_at.clone(),
        draft: updates.draft,
        deleted_at: current.metadata.deleted_at.clone(),
        is_org_issue: current.metadata.is_org_issue,
        org_slug: current.metadata.org_slug.clone(),
        org_display_number: current.metadata.org_display_number,
    }
}

/// Handle planning note transitions when status changes
async fn handle_planning_note_in_content(
    issue_md: &str,
    old_status: &str,
    new_status: &str,
    issue_path: &Path,
) -> std::io::Result<String> {
    let transitioning_to_planning =
        !is_planning_status(old_status) && is_planning_status(new_status);
    let staying_in_planning = is_planning_status(old_status) && is_planning_status(new_status);

    if transitioning_to_planning {
        Ok(add_planning_note(issue_md))
    } else if staying_in_planning {
        let current_issue_md = fs::read_to_string(issue_path.join("issue.md")).await?;
        if has_planning_note(&current_issue_md) {
            Ok(add_planning_note(issue_md))
        } else {
            Ok(issue_md.to_string())
        }
    } else {
        Ok(issue_md.to_string())
    }
}

/// Build Issue struct from update results
fn build_issue_struct(
    issue_number: &str,
    updates: &AppliedIssueUpdates,
    current: &Issue,
    updated_at: &str,
) -> Issue {
    #[allow(deprecated)]
    Issue {
        id: issue_number.to_string(),
        issue_number: issue_number.to_string(),
        title: updates.title.clone(),
        description: updates.description.clone(),
        metadata: IssueMetadataFlat {
            display_number: current.metadata.display_number,
            status: updates.status.clone(),
            priority: updates.priority,
            created_at: current.metadata.created_at.clone(),
            updated_at: updated_at.to_string(),
            custom_fields: updates.custom_fields.clone(),
            compacted: current.metadata.compacted,
            compacted_at: current.metadata.compacted_at.clone(),
            draft: updates.draft,
            deleted_at: current.metadata.deleted_at.clone(),
            is_org_issue: current.metadata.is_org_issue,
            org_slug: current.metadata.org_slug.clone(),
            org_display_number: current.metadata.org_display_number,
        },
    }
}

/// Search for issues by UUID across all tracked projects
/// This is a global search that doesn't require a project_path
pub async fn get_issues_by_uuid(
    uuid: &str,
    projects: &[ProjectInfo],
) -> Result<GetIssuesByUuidResult, IssueCrudError> {
    // Validate that uuid is a valid UUID format
    if !is_uuid(uuid) {
        return Err(IssueCrudError::InvalidIssueFormat(
            "Only UUID format is supported for global search".to_string(),
        ));
    }

    let mut found_issues = Vec::new();
    let mut errors = Vec::new();

    for project in projects {
        // Skip uninitialized projects
        if !project.initialized {
            continue;
        }

        let project_path = Path::new(&project.path);

        // Try to get the issue from this project
        match get_issue(project_path, uuid).await {
            Ok(issue) => {
                let project_name = project.name.clone().unwrap_or_else(|| {
                    project_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| project.path.clone())
                });

                found_issues.push(IssueWithProject {
                    issue,
                    project_path: project.path.clone(),
                    project_name,
                });
            }
            Err(IssueCrudError::IssueNotFound(_)) => {
                // Not an error - issue simply doesn't exist in this project
            }
            Err(IssueCrudError::NotInitialized) => {
                // Skip - project not properly initialized
            }
            Err(e) => {
                // Log non-fatal errors but continue searching
                errors.push(format!("Error searching {}: {}", project.path, e));
            }
        }
    }

    Ok(GetIssuesByUuidResult {
        issues: found_issues,
        errors,
    })
}

/// Update an existing issue
pub async fn update_issue(
    project_path: &Path,
    issue_number: &str,
    options: UpdateIssueOptions,
) -> Result<UpdateIssueResult, IssueCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issue_path = centy_path.join("issues").join(issue_number);

    if !issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    // Read config for priority_levels validation
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    // Read current issue and capture old status for planning note transitions
    let current = read_issue_from_disk(&issue_path, issue_number).await?;
    let old_status = current.metadata.status.clone();
    // Apply updates (clone to preserve current for later use)
    let new_title = options.title.unwrap_or_else(|| current.title.clone());
    let new_description = options
        .description
        .unwrap_or_else(|| current.description.clone());
    let new_status = options
        .status
        .unwrap_or_else(|| current.metadata.status.clone());

    // Validate status and priority
    if let Some(ref config) = config {
        validate_status(&new_status, &config.allowed_states)?;
    }
    // Apply priority update
    let new_priority = if let Some(p) = options.priority {
        validate_priority(p, priority_levels)?;
        p
    } else {
        current.metadata.priority
    };

    // Merge custom fields and build applied updates struct
    let mut new_custom_fields = current.metadata.custom_fields.clone();
    for (key, value) in options.custom_fields {
        new_custom_fields.insert(key, value);
    }
    let updates = AppliedIssueUpdates {
        title: new_title,
        description: new_description,
        status: new_status,
        priority: new_priority,
        custom_fields: new_custom_fields,
        draft: options.draft.unwrap_or(current.metadata.draft),
    };

    // Build updated metadata and generate content with planning note handling
    let updated_metadata = build_updated_metadata(&current, &updates);
    let base_issue_md = generate_issue_md(&updates.title, &updates.description);
    let issue_md =
        handle_planning_note_in_content(&base_issue_md, &old_status, &updates.status, &issue_path)
            .await?;

    // Write files
    fs::write(issue_path.join("issue.md"), &issue_md).await?;
    fs::write(
        issue_path.join("metadata.json"),
        serde_json::to_string_pretty(&updated_metadata)?,
    )
    .await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    // Build result issue struct using helper
    let issue = build_issue_struct(
        issue_number,
        &updates,
        &current,
        &updated_metadata.common.updated_at,
    );

    // Sync to other org projects if this is an org issue
    let sync_results = if issue.metadata.is_org_issue {
        crate::common::sync_update_to_org_projects(&issue, project_path, None).await
    } else {
        Vec::new()
    };

    Ok(UpdateIssueResult {
        issue,
        manifest,
        sync_results,
    })
}

/// Delete an issue
pub async fn delete_issue(
    project_path: &Path,
    issue_number: &str,
) -> Result<DeleteIssueResult, IssueCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issue_path = centy_path.join("issues").join(issue_number);

    if !issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    // Remove the issue directory
    fs::remove_dir_all(&issue_path).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(DeleteIssueResult { manifest })
}

/// Soft-delete an issue (set deleted_at timestamp)
pub async fn soft_delete_issue(
    project_path: &Path,
    issue_number: &str,
) -> Result<SoftDeleteIssueResult, IssueCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issue_path = centy_path.join("issues").join(issue_number);

    if !issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    // Read current metadata
    let metadata_path = issue_path.join("metadata.json");
    let metadata_content = fs::read_to_string(&metadata_path).await?;
    let mut metadata: IssueMetadata = serde_json::from_str(&metadata_content)?;

    // Check if already deleted
    if metadata.deleted_at.is_some() {
        return Err(IssueCrudError::IssueAlreadyDeleted(
            issue_number.to_string(),
        ));
    }

    // Set deleted_at and update updated_at
    let now = now_iso();
    metadata.deleted_at = Some(now.clone());
    metadata.common.updated_at = now;

    // Write updated metadata
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    // Read and return the updated issue
    let issue = read_issue_from_disk(&issue_path, issue_number).await?;

    Ok(SoftDeleteIssueResult { issue, manifest })
}

/// Restore a soft-deleted issue (clear deleted_at timestamp)
pub async fn restore_issue(
    project_path: &Path,
    issue_number: &str,
) -> Result<RestoreIssueResult, IssueCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issue_path = centy_path.join("issues").join(issue_number);

    if !issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    // Read current metadata
    let metadata_path = issue_path.join("metadata.json");
    let metadata_content = fs::read_to_string(&metadata_path).await?;
    let mut metadata: IssueMetadata = serde_json::from_str(&metadata_content)?;

    // Check if actually deleted
    if metadata.deleted_at.is_none() {
        return Err(IssueCrudError::IssueNotDeleted(issue_number.to_string()));
    }

    // Clear deleted_at and update updated_at
    metadata.deleted_at = None;
    metadata.common.updated_at = now_iso();

    // Write updated metadata
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    // Read and return the restored issue
    let issue = read_issue_from_disk(&issue_path, issue_number).await?;

    Ok(RestoreIssueResult { issue, manifest })
}

/// Move an issue to another project
///
/// The issue keeps its UUID (preserving cross-project references) but gets
/// a new display number in the target project. Assets are copied to the target.
///
/// # Arguments
/// * `options` - Move options specifying source, target, and issue ID
///
/// # Returns
/// The moved issue with updated display number, plus both manifests
pub async fn move_issue(options: MoveIssueOptions) -> Result<MoveIssueResult, IssueCrudError> {
    // Verify not same project
    if options.source_project_path == options.target_project_path {
        return Err(IssueCrudError::SameProject);
    }

    // Validate source project is initialized
    let mut source_manifest = read_manifest(&options.source_project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    // Validate target project is initialized
    let mut target_manifest = read_manifest(&options.target_project_path)
        .await?
        .ok_or(IssueCrudError::TargetNotInitialized)?;

    // Read source issue
    let source_centy = get_centy_path(&options.source_project_path);
    let source_issue_path = source_centy.join("issues").join(&options.issue_id);

    if !source_issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(options.issue_id.clone()));
    }

    let source_issue = read_issue_from_disk(&source_issue_path, &options.issue_id).await?;
    let old_display_number = source_issue.metadata.display_number;

    // Read target config for priority validation
    let target_config = read_config(&options.target_project_path)
        .await
        .ok()
        .flatten();
    let target_priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);

    // Validate priority is within target's range
    if source_issue.metadata.priority > target_priority_levels {
        return Err(IssueCrudError::InvalidPriorityInTarget(
            source_issue.metadata.priority,
        ));
    }

    // Status validation: reject if status is not valid in target project
    if let Some(ref config) = target_config {
        validate_status(&source_issue.metadata.status, &config.allowed_states)?;
    }

    // Get next display number in target project
    let target_centy = get_centy_path(&options.target_project_path);
    let target_issues_path = target_centy.join("issues");
    fs::create_dir_all(&target_issues_path).await?;
    let new_display_number = get_next_display_number(&target_issues_path).await?;

    // Create target issue folder (same UUID)
    let target_issue_path = target_issues_path.join(&options.issue_id);
    fs::create_dir_all(&target_issue_path).await?;
    fs::create_dir_all(target_issue_path.join("assets")).await?;

    // Copy issue.md
    fs::copy(
        source_issue_path.join("issue.md"),
        target_issue_path.join("issue.md"),
    )
    .await?;

    // Read, update, and write metadata with new display number
    let metadata_content = fs::read_to_string(source_issue_path.join("metadata.json")).await?;
    let mut metadata: IssueMetadata = serde_json::from_str(&metadata_content)?;
    metadata.common.display_number = new_display_number;
    metadata.common.updated_at = now_iso();
    fs::write(
        target_issue_path.join("metadata.json"),
        serde_json::to_string_pretty(&metadata)?,
    )
    .await?;

    // Copy assets
    let source_assets_path = source_issue_path.join("assets");
    let target_assets_path = target_issue_path.join("assets");
    copy_assets_folder(&source_assets_path, &target_assets_path)
        .await
        .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;

    // Delete from source project
    fs::remove_dir_all(&source_issue_path).await?;

    // Update both manifests
    update_manifest_timestamp(&mut source_manifest);
    update_manifest_timestamp(&mut target_manifest);
    write_manifest(&options.source_project_path, &source_manifest).await?;
    write_manifest(&options.target_project_path, &target_manifest).await?;

    // Read the moved issue from target
    let moved_issue = read_issue_from_disk(&target_issue_path, &options.issue_id).await?;

    Ok(MoveIssueResult {
        issue: moved_issue,
        old_display_number,
        source_manifest,
        target_manifest,
    })
}

/// Duplicate an issue to the same or different project
///
/// Creates a copy of the issue with a new UUID and display number.
/// Assets are copied to the new issue folder.
///
/// # Arguments
/// * `options` - Duplicate options specifying source, target, issue ID, and optional new title
///
/// # Returns
/// The new duplicate issue with the original issue ID for reference
pub async fn duplicate_issue(
    options: DuplicateIssueOptions,
) -> Result<DuplicateIssueResult, IssueCrudError> {
    // Validate source project is initialized
    read_manifest(&options.source_project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    // Validate target project is initialized
    let mut target_manifest = read_manifest(&options.target_project_path)
        .await?
        .ok_or(IssueCrudError::TargetNotInitialized)?;

    // Read source issue
    let source_centy = get_centy_path(&options.source_project_path);
    let source_issue_path = source_centy.join("issues").join(&options.issue_id);

    if !source_issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(options.issue_id.clone()));
    }

    let source_issue = read_issue_from_disk(&source_issue_path, &options.issue_id).await?;

    // Read target config for priority validation (if different project)
    if options.source_project_path != options.target_project_path {
        let target_config = read_config(&options.target_project_path)
            .await
            .ok()
            .flatten();
        let target_priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);

        // Validate priority is within target's range
        if source_issue.metadata.priority > target_priority_levels {
            return Err(IssueCrudError::InvalidPriorityInTarget(
                source_issue.metadata.priority,
            ));
        }

        // Status validation: reject if status is not valid in target project
        if let Some(ref config) = target_config {
            validate_status(&source_issue.metadata.status, &config.allowed_states)?;
        }
    }

    // Generate new UUID for the duplicate
    let new_id = generate_issue_id();

    // Get next display number in target project
    let target_centy = get_centy_path(&options.target_project_path);
    let target_issues_path = target_centy.join("issues");
    fs::create_dir_all(&target_issues_path).await?;
    let new_display_number = get_next_display_number(&target_issues_path).await?;

    // Create new issue folder
    let new_issue_path = target_issues_path.join(&new_id);
    fs::create_dir_all(&new_issue_path).await?;
    fs::create_dir_all(new_issue_path.join("assets")).await?;

    // Prepare new title
    let new_title = options
        .new_title
        .unwrap_or_else(|| format!("Copy of {}", source_issue.title));

    // Create new issue.md
    let mut issue_md = generate_issue_md(&new_title, &source_issue.description);

    // Add planning note if source issue is in planning state
    if is_planning_status(&source_issue.metadata.status) {
        issue_md = add_planning_note(&issue_md);
    }

    fs::write(new_issue_path.join("issue.md"), &issue_md).await?;

    // Create new metadata with fresh timestamps
    // Note: Duplicating an org issue creates a local copy, not an org issue
    let new_metadata = IssueMetadata {
        common: crate::common::CommonMetadata {
            display_number: new_display_number,
            status: source_issue.metadata.status.clone(),
            priority: source_issue.metadata.priority,
            created_at: now_iso(),
            updated_at: now_iso(),
            custom_fields: source_issue
                .metadata
                .custom_fields
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        },
        compacted: false, // Reset compacted status for new issue
        compacted_at: None,
        draft: source_issue.metadata.draft, // Preserve draft status
        deleted_at: None,                   // New duplicate is not deleted
        is_org_issue: false,                // Duplicate is always a local copy
        org_slug: None,
        org_display_number: None,
    };
    fs::write(
        new_issue_path.join("metadata.json"),
        serde_json::to_string_pretty(&new_metadata)?,
    )
    .await?;

    // Copy assets
    let source_assets_path = source_issue_path.join("assets");
    let target_assets_path = new_issue_path.join("assets");
    copy_assets_folder(&source_assets_path, &target_assets_path)
        .await
        .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;

    // Update target manifest
    update_manifest_timestamp(&mut target_manifest);
    write_manifest(&options.target_project_path, &target_manifest).await?;

    // Read the new issue
    let new_issue = read_issue_from_disk(&new_issue_path, &new_id).await?;

    Ok(DuplicateIssueResult {
        issue: new_issue,
        original_issue_id: options.issue_id,
        manifest: target_manifest,
    })
}

/// Read an issue from disk
async fn read_issue_from_disk(
    issue_path: &Path,
    issue_number: &str,
) -> Result<Issue, IssueCrudError> {
    let issue_md_path = issue_path.join("issue.md");
    let metadata_path = issue_path.join("metadata.json");

    if !issue_md_path.exists() || !metadata_path.exists() {
        return Err(IssueCrudError::InvalidIssueFormat(format!(
            "Issue {issue_number} is missing required files"
        )));
    }

    // Read issue.md
    let issue_md = fs::read_to_string(&issue_md_path).await?;
    let (title, description) = parse_issue_md(&issue_md);

    // Read metadata (serde will auto-migrate string priorities to numbers)
    let metadata_content = fs::read_to_string(&metadata_path).await?;
    let metadata: IssueMetadata = serde_json::from_str(&metadata_content)?;

    // Convert custom fields to strings
    let custom_fields: HashMap<String, String> = metadata
        .common
        .custom_fields
        .into_iter()
        .map(|(k, v)| {
            let str_val = match v {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            (k, str_val)
        })
        .collect();

    #[allow(deprecated)]
    Ok(Issue {
        id: issue_number.to_string(),
        issue_number: issue_number.to_string(), // Legacy field
        title,
        description,
        metadata: IssueMetadataFlat {
            display_number: metadata.common.display_number,
            status: metadata.common.status,
            priority: metadata.common.priority,
            created_at: metadata.common.created_at,
            updated_at: metadata.common.updated_at,
            custom_fields,
            compacted: metadata.compacted,
            compacted_at: metadata.compacted_at,
            draft: metadata.draft,
            deleted_at: metadata.deleted_at,
            is_org_issue: metadata.is_org_issue,
            org_slug: metadata.org_slug,
            org_display_number: metadata.org_display_number,
        },
    })
}

/// Parse issue.md content to extract title and description
/// Handles the planning note by skipping it if present at the start
fn parse_issue_md(content: &str) -> (String, String) {
    // Remove planning note if present before parsing
    let content = remove_planning_note(content);
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return (String::new(), String::new());
    }

    // Find the title line (should start with #)
    let mut title_idx = 0;
    for (idx, line) in lines.iter().enumerate() {
        if line.starts_with('#') {
            title_idx = idx;
            break;
        }
    }

    // Extract title
    let title = lines
        .get(title_idx)
        .map(|line| line.strip_prefix('#').map_or(*line, str::trim))
        .unwrap_or("")
        .to_string();

    // Rest is description (skip empty lines after title)
    let description_lines: Vec<&str> = lines[(title_idx + 1)..]
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect();

    let description = description_lines.join("\n").trim_end().to_string();

    (title, description)
}

/// Generate the issue markdown content
fn generate_issue_md(title: &str, description: &str) -> String {
    if description.is_empty() {
        format!("# {title}\n")
    } else {
        format!("# {title}\n\n{description}\n")
    }
}

// =============================================================================
// OrgSyncable implementation for Issue
// =============================================================================

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

/// Create an issue in a specific project (used for org issue sync).
///
/// This function does NOT trigger recursive org sync to prevent infinite loops.
/// If the issue already exists in the target project, it is skipped.
async fn create_issue_in_project(
    project_path: &Path,
    issue_id: &str,
    title: &str,
    description: &str,
    source_metadata: &IssueMetadataFlat,
    org_slug: &str,
) -> Result<(), OrgSyncError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?
        .ok_or_else(|| OrgSyncError::SyncFailed("Target project not initialized".to_string()))?;

    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    let issue_path = issues_path.join(issue_id);

    // Skip if already exists (don't overwrite local customizations)
    if issue_path.exists() {
        return Ok(());
    }

    // Ensure directories exist
    fs::create_dir_all(&issue_path).await?;
    fs::create_dir_all(issue_path.join("assets")).await?;

    // Get local display number for this project
    let local_display_number = get_next_display_number(&issues_path)
        .await
        .map_err(|e| OrgSyncError::SyncFailed(e.to_string()))?;

    // Create metadata with org fields
    let metadata = IssueMetadata::new_org_issue(
        local_display_number,
        source_metadata.org_display_number.unwrap_or(0),
        source_metadata.status.clone(),
        source_metadata.priority,
        org_slug,
        source_metadata
            .custom_fields
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect(),
        source_metadata.draft,
    );

    // Write issue.md
    let issue_md = generate_issue_md(title, description);
    fs::write(issue_path.join("issue.md"), format_markdown(&issue_md)).await?;

    // Write metadata
    fs::write(
        issue_path.join("metadata.json"),
        serde_json::to_string_pretty(&metadata)?,
    )
    .await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?;

    Ok(())
}

/// Update or create an issue in a project (for org sync on update).
///
/// If the issue doesn't exist, it will be created.
/// If the issue exists, it will be updated with the new content while
/// preserving local fields like `display_number` and `created_at`.
async fn update_or_create_issue_in_project(
    project_path: &Path,
    issue_id: &str,
    title: &str,
    description: &str,
    source_metadata: &IssueMetadataFlat,
    org_slug: &str,
) -> Result<(), OrgSyncError> {
    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    let issue_path = issues_path.join(issue_id);

    if !issue_path.exists() {
        // Issue doesn't exist locally - create it
        return create_issue_in_project(
            project_path,
            issue_id,
            title,
            description,
            source_metadata,
            org_slug,
        )
        .await;
    }

    // Update existing issue
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?
        .ok_or_else(|| OrgSyncError::SyncFailed("Target project not initialized".to_string()))?;

    // Read existing metadata to preserve local fields
    let existing_metadata_str = fs::read_to_string(issue_path.join("metadata.json")).await?;
    let existing: IssueMetadata = serde_json::from_str(&existing_metadata_str)?;

    // Update metadata preserving local fields
    let updated_metadata = IssueMetadata {
        common: crate::common::CommonMetadata {
            display_number: existing.common.display_number, // Keep local display number
            status: source_metadata.status.clone(),
            priority: source_metadata.priority,
            created_at: existing.common.created_at, // Preserve original created_at
            updated_at: now_iso(),
            custom_fields: source_metadata
                .custom_fields
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        },
        compacted: existing.compacted, // Preserve local compaction state
        compacted_at: existing.compacted_at,
        draft: source_metadata.draft,
        deleted_at: source_metadata.deleted_at.clone(),
        is_org_issue: true,
        org_slug: Some(org_slug.to_string()),
        org_display_number: source_metadata.org_display_number,
    };

    // Write updated issue.md
    let issue_md = generate_issue_md(title, description);
    fs::write(issue_path.join("issue.md"), format_markdown(&issue_md)).await?;

    // Write updated metadata
    fs::write(
        issue_path.join("metadata.json"),
        serde_json::to_string_pretty(&updated_metadata)?,
    )
    .await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issue_md_with_description() {
        let content = "# My Issue Title\n\nThis is the description.\nWith multiple lines.";
        let (title, description) = parse_issue_md(content);
        assert_eq!(title, "My Issue Title");
        assert_eq!(
            description,
            "This is the description.\nWith multiple lines."
        );
    }

    #[test]
    fn test_parse_issue_md_title_only() {
        let content = "# My Issue Title\n";
        let (title, description) = parse_issue_md(content);
        assert_eq!(title, "My Issue Title");
        assert_eq!(description, "");
    }

    #[test]
    fn test_parse_issue_md_empty() {
        let content = "";
        let (title, description) = parse_issue_md(content);
        assert_eq!(title, "");
        assert_eq!(description, "");
    }
}
