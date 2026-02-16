use super::assets::copy_assets_folder;
use super::id::{is_uuid, is_valid_issue_file, is_valid_issue_folder};
use super::metadata::{IssueFrontmatter, IssueMetadata};
use super::planning::{
    add_planning_note, has_planning_note, is_planning_status, remove_planning_note,
};
use super::priority::{validate_priority, PriorityError};
use super::reconcile::{get_next_display_number, reconcile_display_numbers, ReconcileError};
use super::status::{validate_status, StatusError};
use crate::common::{
    generate_frontmatter, parse_frontmatter, FrontmatterError, OrgSyncError, OrgSyncable,
};
use crate::config::read_config;
use crate::link::{read_links, write_links};
use crate::manifest::{read_manifest, update_manifest, write_manifest, CentyManifest};
use crate::registry::ProjectInfo;
use crate::utils::{format_markdown, get_centy_path, now_iso};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tracing::debug;

#[derive(Error, Debug)]
pub enum IssueCrudError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML frontmatter error: {0}")]
    FrontmatterError(#[from] FrontmatterError),

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

/// Get a single issue by its number
pub async fn get_issue(project_path: &Path, issue_number: &str) -> Result<Issue, IssueCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");

    // Try new format first: {uuid}.md file
    let issue_file_path = issues_path.join(format!("{issue_number}.md"));
    if issue_file_path.exists() {
        return read_issue_from_frontmatter(&issue_file_path, issue_number).await;
    }

    // Fallback to old format: {uuid}/ folder with issue.md and metadata.json
    // Auto-migrate to new format
    let issue_folder_path = issues_path.join(issue_number);
    if issue_folder_path.exists() {
        return migrate_issue_to_new_format(&issues_path, &issue_folder_path, issue_number).await;
    }

    Err(IssueCrudError::IssueNotFound(issue_number.to_string()))
}

/// Migrate an issue from legacy folder format to new frontmatter format
///
/// This function:
/// 1. Reads the issue from the old format
/// 2. Writes it to the new format (.md file with frontmatter)
/// 3. Migrates assets from {id}/assets/ to assets/{id}/
/// 4. Migrates links from {id}/links.json to links/{id}/links.json
/// 5. Deletes the old folder
async fn migrate_issue_to_new_format(
    issues_path: &Path,
    issue_folder_path: &Path,
    issue_id: &str,
) -> Result<Issue, IssueCrudError> {
    debug!("Auto-migrating issue {} to new format", issue_id);

    // Read from legacy format
    let issue = read_issue_from_legacy_folder(issue_folder_path, issue_id).await?;

    // Build frontmatter from metadata
    let frontmatter = IssueFrontmatter {
        display_number: issue.metadata.display_number,
        status: issue.metadata.status.clone(),
        priority: issue.metadata.priority,
        created_at: issue.metadata.created_at.clone(),
        updated_at: issue.metadata.updated_at.clone(),
        draft: issue.metadata.draft,

        deleted_at: issue.metadata.deleted_at.clone(),
        is_org_issue: issue.metadata.is_org_issue,
        org_slug: issue.metadata.org_slug.clone(),
        org_display_number: issue.metadata.org_display_number,
        custom_fields: issue.metadata.custom_fields.clone(),
    };

    // Generate body with planning note if in planning status
    let body =
        if is_planning_status(&issue.metadata.status) && !has_planning_note(&issue.description) {
            add_planning_note(&issue.description)
        } else {
            issue.description.clone()
        };

    // Write new format file
    let issue_content = generate_frontmatter(&frontmatter, &issue.title, &body);
    let new_issue_file = issues_path.join(format!("{issue_id}.md"));
    fs::write(&new_issue_file, format_markdown(&issue_content)).await?;

    // Migrate assets: {id}/assets/ -> assets/{id}/
    let old_assets_path = issue_folder_path.join("assets");
    if old_assets_path.exists() && old_assets_path.is_dir() {
        let new_assets_path = issues_path.join("assets").join(issue_id);
        fs::create_dir_all(&new_assets_path).await?;

        // Move all files from old to new assets directory
        let mut entries = fs::read_dir(&old_assets_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let old_file = entry.path();
            let new_file = new_assets_path.join(&file_name);
            fs::rename(&old_file, &new_file).await?;
        }
        debug!("Migrated assets for issue {}", issue_id);
    }

    // Migrate links: {id}/links.json -> links/{id}/links.json
    let old_links = read_links(issue_folder_path).await?;
    if !old_links.links.is_empty() {
        // write_links will write to the new location (links/{id}/links.json)
        // and clean up the old location
        write_links(issue_folder_path, &old_links).await?;
        debug!("Migrated links for issue {}", issue_id);
    }

    // Delete old folder
    fs::remove_dir_all(issue_folder_path).await?;
    debug!("Deleted old issue folder for {}", issue_id);

    Ok(issue)
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
        let file_type = entry.file_type().await?;
        if let Some(name) = entry.file_name().to_str() {
            let read_result = if file_type.is_file() && is_valid_issue_file(name) {
                // New format: {uuid}.md file
                let issue_id = name.trim_end_matches(".md");
                read_issue_from_frontmatter(&entry.path(), issue_id).await
            } else if file_type.is_dir() && is_valid_issue_folder(name) {
                // Old format: {uuid}/ folder - auto-migrate
                migrate_issue_to_new_format(&issues_path, &entry.path(), name).await
            } else {
                continue;
            };

            match read_result {
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
        let file_type = entry.file_type().await?;
        if let Some(name) = entry.file_name().to_str() {
            // Try new format: {uuid}.md file
            if file_type.is_file() && is_valid_issue_file(name) {
                if let Ok(content) = fs::read_to_string(entry.path()).await {
                    if let Ok((frontmatter, _, _)) = parse_frontmatter::<IssueFrontmatter>(&content)
                    {
                        if frontmatter.display_number == display_number {
                            let issue_id = name.trim_end_matches(".md");
                            return read_issue_from_frontmatter(&entry.path(), issue_id).await;
                        }
                    }
                }
            }
            // Try old format: {uuid}/ folder - auto-migrate
            else if file_type.is_dir() && is_valid_issue_folder(name) {
                let metadata_path = entry.path().join("metadata.json");
                if !metadata_path.exists() {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&metadata_path).await {
                    if let Ok(metadata) = serde_json::from_str::<IssueMetadata>(&content) {
                        if metadata.common.display_number == display_number {
                            return migrate_issue_to_new_format(&issues_path, &entry.path(), name)
                                .await;
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
    let issues_path = centy_path.join("issues");

    // Determine if this is new format (file) or old format (folder)
    let issue_file_path = issues_path.join(format!("{issue_number}.md"));
    let issue_folder_path = issues_path.join(issue_number);
    let is_new_format = issue_file_path.exists();
    let is_old_format = issue_folder_path.exists();

    if !is_new_format && !is_old_format {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    // Read config for priority_levels validation
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    // Read current issue and capture old status for planning note transitions
    let current = if is_new_format {
        read_issue_from_frontmatter(&issue_file_path, issue_number).await?
    } else {
        read_issue_from_legacy_folder(&issue_folder_path, issue_number).await?
    };
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

    // Build updated frontmatter and generate content with planning note handling
    let updated_metadata = build_updated_metadata(&current, &updates);
    let frontmatter =
        IssueFrontmatter::from_metadata(&updated_metadata, updates.custom_fields.clone());

    // Handle planning note - add to description (body) if transitioning to planning status
    let body = if is_planning_status(&old_status) && is_planning_status(&updates.status) {
        // Check if current content has planning note
        let current_content = if is_new_format {
            fs::read_to_string(&issue_file_path).await?
        } else {
            fs::read_to_string(issue_folder_path.join("issue.md")).await?
        };
        if has_planning_note(&current_content) {
            add_planning_note(&updates.description)
        } else {
            updates.description.clone()
        }
    } else if !is_planning_status(&old_status) && is_planning_status(&updates.status) {
        // Transitioning to planning status - add note
        add_planning_note(&updates.description)
    } else {
        // Not in planning status - no note
        updates.description.clone()
    };

    // Write in new format
    let issue_content = generate_frontmatter(&frontmatter, &updates.title, &body);
    fs::write(&issue_file_path, &issue_content).await?;

    // If upgrading from old format, remove old folder (but keep assets)
    if is_old_format && !is_new_format {
        // Move assets to new location first
        let old_assets_path = issue_folder_path.join("assets");
        let new_assets_path = issues_path.join("assets").join(issue_number);
        if old_assets_path.exists() {
            fs::create_dir_all(&new_assets_path).await?;
            copy_assets_folder(&old_assets_path, &new_assets_path)
                .await
                .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;
        }
        // Remove old folder
        fs::remove_dir_all(&issue_folder_path).await?;
    }

    // Update manifest timestamp
    update_manifest(&mut manifest);
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
    let issues_path = centy_path.join("issues");
    let issue_file_path = issues_path.join(format!("{issue_number}.md"));
    let issue_folder_path = issues_path.join(issue_number);
    let assets_path = issues_path.join("assets").join(issue_number);

    let exists = issue_file_path.exists() || issue_folder_path.exists();
    if !exists {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    // Remove the issue file (new format)
    if issue_file_path.exists() {
        fs::remove_file(&issue_file_path).await?;
    }

    // Remove the issue directory (old format)
    if issue_folder_path.exists() {
        fs::remove_dir_all(&issue_folder_path).await?;
    }

    // Remove assets directory (new format location)
    if assets_path.exists() {
        fs::remove_dir_all(&assets_path).await?;
    }

    // Update manifest timestamp
    update_manifest(&mut manifest);
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
    let issues_path = centy_path.join("issues");
    let issue_file_path = issues_path.join(format!("{issue_number}.md"));
    let issue_folder_path = issues_path.join(issue_number);

    // Determine format and read current issue
    let (is_new_format, current) = if issue_file_path.exists() {
        (
            true,
            read_issue_from_frontmatter(&issue_file_path, issue_number).await?,
        )
    } else if issue_folder_path.exists() {
        (
            false,
            read_issue_from_legacy_folder(&issue_folder_path, issue_number).await?,
        )
    } else {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    };

    // Check if already deleted
    if current.metadata.deleted_at.is_some() {
        return Err(IssueCrudError::IssueAlreadyDeleted(
            issue_number.to_string(),
        ));
    }

    // Set deleted_at and update updated_at
    let now = now_iso();
    let frontmatter = IssueFrontmatter {
        display_number: current.metadata.display_number,
        status: current.metadata.status.clone(),
        priority: current.metadata.priority,
        created_at: current.metadata.created_at.clone(),
        updated_at: now.clone(),
        draft: current.metadata.draft,
        deleted_at: Some(now),
        is_org_issue: current.metadata.is_org_issue,
        org_slug: current.metadata.org_slug.clone(),
        org_display_number: current.metadata.org_display_number,
        custom_fields: current.metadata.custom_fields.clone(),
    };

    // Write in new format
    let issue_content = generate_frontmatter(&frontmatter, &current.title, &current.description);
    fs::write(&issue_file_path, &issue_content).await?;

    // If upgrading from old format, migrate assets and remove old folder
    if !is_new_format {
        let old_assets_path = issue_folder_path.join("assets");
        let new_assets_path = issues_path.join("assets").join(issue_number);
        if old_assets_path.exists() {
            fs::create_dir_all(&new_assets_path).await?;
            copy_assets_folder(&old_assets_path, &new_assets_path)
                .await
                .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;
        }
        fs::remove_dir_all(&issue_folder_path).await?;
    }

    // Update manifest timestamp
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    // Read and return the updated issue
    let issue = read_issue_from_frontmatter(&issue_file_path, issue_number).await?;

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
    let issues_path = centy_path.join("issues");
    let issue_file_path = issues_path.join(format!("{issue_number}.md"));
    let issue_folder_path = issues_path.join(issue_number);

    // Determine format and read current issue
    let (is_new_format, current) = if issue_file_path.exists() {
        (
            true,
            read_issue_from_frontmatter(&issue_file_path, issue_number).await?,
        )
    } else if issue_folder_path.exists() {
        (
            false,
            read_issue_from_legacy_folder(&issue_folder_path, issue_number).await?,
        )
    } else {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    };

    // Check if actually deleted
    if current.metadata.deleted_at.is_none() {
        return Err(IssueCrudError::IssueNotDeleted(issue_number.to_string()));
    }

    // Clear deleted_at and update updated_at
    let frontmatter = IssueFrontmatter {
        display_number: current.metadata.display_number,
        status: current.metadata.status.clone(),
        priority: current.metadata.priority,
        created_at: current.metadata.created_at.clone(),
        updated_at: now_iso(),
        draft: current.metadata.draft,
        deleted_at: None,
        is_org_issue: current.metadata.is_org_issue,
        org_slug: current.metadata.org_slug.clone(),
        org_display_number: current.metadata.org_display_number,
        custom_fields: current.metadata.custom_fields.clone(),
    };

    // Write in new format
    let issue_content = generate_frontmatter(&frontmatter, &current.title, &current.description);
    fs::write(&issue_file_path, &issue_content).await?;

    // If upgrading from old format, migrate assets and remove old folder
    if !is_new_format {
        let old_assets_path = issue_folder_path.join("assets");
        let new_assets_path = issues_path.join("assets").join(issue_number);
        if old_assets_path.exists() {
            fs::create_dir_all(&new_assets_path).await?;
            copy_assets_folder(&old_assets_path, &new_assets_path)
                .await
                .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;
        }
        fs::remove_dir_all(&issue_folder_path).await?;
    }

    // Update manifest timestamp
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    // Read and return the restored issue
    let issue = read_issue_from_frontmatter(&issue_file_path, issue_number).await?;

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

    // Read source issue (supports both formats)
    let source_centy = get_centy_path(&options.source_project_path);
    let source_issues_path = source_centy.join("issues");
    let source_file_path = source_issues_path.join(format!("{}.md", &options.issue_id));
    let source_folder_path = source_issues_path.join(&options.issue_id);

    let (source_is_new_format, source_issue) = if source_file_path.exists() {
        (
            true,
            read_issue_from_frontmatter(&source_file_path, &options.issue_id).await?,
        )
    } else if source_folder_path.exists() {
        (
            false,
            read_issue_from_legacy_folder(&source_folder_path, &options.issue_id).await?,
        )
    } else {
        return Err(IssueCrudError::IssueNotFound(options.issue_id.clone()));
    };
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

    // Create frontmatter with updated display number
    let frontmatter = IssueFrontmatter {
        display_number: new_display_number,
        status: source_issue.metadata.status.clone(),
        priority: source_issue.metadata.priority,
        created_at: source_issue.metadata.created_at.clone(),
        updated_at: now_iso(),
        draft: source_issue.metadata.draft,

        deleted_at: source_issue.metadata.deleted_at.clone(),
        is_org_issue: source_issue.metadata.is_org_issue,
        org_slug: source_issue.metadata.org_slug.clone(),
        org_display_number: source_issue.metadata.org_display_number,
        custom_fields: source_issue.metadata.custom_fields.clone(),
    };

    // Write to target in new format
    let target_issue_file = target_issues_path.join(format!("{}.md", &options.issue_id));
    let issue_content =
        generate_frontmatter(&frontmatter, &source_issue.title, &source_issue.description);
    fs::write(&target_issue_file, &issue_content).await?;

    // Copy assets to new location
    let source_assets_path = if source_is_new_format {
        source_issues_path.join("assets").join(&options.issue_id)
    } else {
        source_folder_path.join("assets")
    };
    let target_assets_path = target_issues_path.join("assets").join(&options.issue_id);
    if source_assets_path.exists() {
        fs::create_dir_all(&target_assets_path).await?;
        copy_assets_folder(&source_assets_path, &target_assets_path)
            .await
            .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;
    }

    // Delete from source project
    if source_is_new_format {
        fs::remove_file(&source_file_path).await?;
        if source_assets_path.exists() {
            fs::remove_dir_all(&source_assets_path).await?;
        }
    } else {
        fs::remove_dir_all(&source_folder_path).await?;
    }

    // Update both manifests
    update_manifest(&mut source_manifest);
    update_manifest(&mut target_manifest);
    write_manifest(&options.source_project_path, &source_manifest).await?;
    write_manifest(&options.target_project_path, &target_manifest).await?;

    // Read the moved issue from target
    let moved_issue = read_issue_from_frontmatter(&target_issue_file, &options.issue_id).await?;

    Ok(MoveIssueResult {
        issue: moved_issue,
        old_display_number,
        source_manifest,
        target_manifest,
    })
}

/// Read an issue from disk
/// Read an issue from the new frontmatter format ({uuid}.md file)
async fn read_issue_from_frontmatter(
    issue_file_path: &Path,
    issue_id: &str,
) -> Result<Issue, IssueCrudError> {
    let content = fs::read_to_string(issue_file_path).await?;
    let (frontmatter, title, body): (IssueFrontmatter, String, String) =
        parse_frontmatter(&content)?;

    // Remove planning note from body if present
    let description = remove_planning_note(&body);

    #[allow(deprecated)]
    Ok(Issue {
        id: issue_id.to_string(),
        issue_number: issue_id.to_string(), // Legacy field
        title,
        description,
        metadata: IssueMetadataFlat {
            display_number: frontmatter.display_number,
            status: frontmatter.status,
            priority: frontmatter.priority,
            created_at: frontmatter.created_at,
            updated_at: frontmatter.updated_at,
            custom_fields: frontmatter.custom_fields,
            draft: frontmatter.draft,
            deleted_at: frontmatter.deleted_at,
            is_org_issue: frontmatter.is_org_issue,
            org_slug: frontmatter.org_slug,
            org_display_number: frontmatter.org_display_number,
        },
    })
}

/// Read an issue from the legacy folder format ({uuid}/ with issue.md + metadata.json)
async fn read_issue_from_legacy_folder(
    issue_folder_path: &Path,
    issue_id: &str,
) -> Result<Issue, IssueCrudError> {
    let issue_md_path = issue_folder_path.join("issue.md");
    let metadata_path = issue_folder_path.join("metadata.json");

    if !issue_md_path.exists() || !metadata_path.exists() {
        return Err(IssueCrudError::InvalidIssueFormat(format!(
            "Issue {issue_id} is missing required files"
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
        id: issue_id.to_string(),
        issue_number: issue_id.to_string(), // Legacy field
        title,
        description,
        metadata: IssueMetadataFlat {
            display_number: metadata.common.display_number,
            status: metadata.common.status,
            priority: metadata.common.priority,
            created_at: metadata.common.created_at,
            updated_at: metadata.common.updated_at,
            custom_fields,
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
    let description_lines: Vec<&str> = lines
        .get(title_idx.saturating_add(1)..)
        .unwrap_or(&[])
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
    let issue_file_path = issues_path.join(format!("{issue_id}.md"));
    let issue_folder_path = issues_path.join(issue_id);

    // Skip if already exists (don't overwrite local customizations)
    if issue_file_path.exists() || issue_folder_path.exists() {
        return Ok(());
    }

    // Ensure issues directory exists
    fs::create_dir_all(&issues_path).await?;

    // Get local display number for this project
    let local_display_number = get_next_display_number(&issues_path)
        .await
        .map_err(|e| OrgSyncError::SyncFailed(e.to_string()))?;

    // Create frontmatter with org fields
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

    // Write issue in frontmatter format
    let issue_content = generate_frontmatter(&frontmatter, title, description);
    fs::write(&issue_file_path, format_markdown(&issue_content)).await?;

    // Update manifest timestamp
    update_manifest(&mut manifest);
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
    let issue_file_path = issues_path.join(format!("{issue_id}.md"));
    let issue_folder_path = issues_path.join(issue_id);

    // Check if issue exists in either format
    let (is_new_format, existing_issue) = if issue_file_path.exists() {
        match read_issue_from_frontmatter(&issue_file_path, issue_id).await {
            Ok(issue) => (true, Some(issue)),
            Err(_) => (true, None),
        }
    } else if issue_folder_path.exists() {
        match read_issue_from_legacy_folder(&issue_folder_path, issue_id).await {
            Ok(issue) => (false, Some(issue)),
            Err(_) => (false, None),
        }
    } else {
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
    };

    let existing = match existing_issue {
        Some(issue) => issue,
        None => {
            // Failed to read, create new
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
    };

    // Update existing issue
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|e| OrgSyncError::ManifestError(e.to_string()))?
        .ok_or_else(|| OrgSyncError::SyncFailed("Target project not initialized".to_string()))?;

    // Create updated frontmatter preserving local fields
    let frontmatter = IssueFrontmatter {
        display_number: existing.metadata.display_number, // Keep local display number
        status: source_metadata.status.clone(),
        priority: source_metadata.priority,
        created_at: existing.metadata.created_at.clone(), // Preserve original created_at
        updated_at: now_iso(),
        draft: source_metadata.draft,
        deleted_at: source_metadata.deleted_at.clone(),
        is_org_issue: true,
        org_slug: Some(org_slug.to_string()),
        org_display_number: source_metadata.org_display_number,
        custom_fields: source_metadata.custom_fields.clone(),
    };

    // Write updated issue in frontmatter format
    let issue_content = generate_frontmatter(&frontmatter, title, description);
    fs::write(&issue_file_path, format_markdown(&issue_content)).await?;

    // If upgrading from old format, migrate assets and remove old folder
    if !is_new_format {
        let old_assets_path = issue_folder_path.join("assets");
        let new_assets_path = issues_path.join("assets").join(issue_id);
        if old_assets_path.exists() {
            fs::create_dir_all(&new_assets_path).await?;
            let _ = copy_assets_folder(&old_assets_path, &new_assets_path).await;
        }
        let _ = fs::remove_dir_all(&issue_folder_path).await;
    }

    // Update manifest timestamp
    update_manifest(&mut manifest);
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
