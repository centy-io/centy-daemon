use crate::config::read_config;
use crate::manifest::{
    read_manifest, write_manifest, update_manifest_timestamp, CentyManifest,
};
use crate::registry::ProjectInfo;
use crate::utils::{get_centy_path, now_iso};
use crate::item::validation::priority::{validate_priority, PriorityError};
use super::id::{is_uuid, is_valid_pr_folder};
use super::metadata::PrMetadata;
use super::reconcile::{reconcile_pr_display_numbers, ReconcileError};
use super::status::{default_pr_statuses, validate_pr_status};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum PrCrudError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("PR {0} not found")]
    PrNotFound(String),

    #[error("PR with display number {0} not found")]
    PrDisplayNumberNotFound(u32),

    #[error("PR {0} is not soft-deleted")]
    PrNotDeleted(String),

    #[error("PR {0} is already soft-deleted")]
    PrAlreadyDeleted(String),

    #[error("Invalid PR format: {0}")]
    InvalidPrFormat(String),

    #[error("Invalid priority: {0}")]
    InvalidPriority(#[from] PriorityError),

    #[error("Reconcile error: {0}")]
    ReconcileError(#[from] ReconcileError),
}

/// Full PR data
#[derive(Debug, Clone)]
pub struct PullRequest {
    /// UUID-based PR ID (folder name)
    pub id: String,
    pub title: String,
    pub description: String,
    pub metadata: PrMetadataFlat,
}

/// Flattened metadata for API responses
#[derive(Debug, Clone)]
pub struct PrMetadataFlat {
    /// Human-readable display number (1, 2, 3...)
    pub display_number: u32,
    pub status: String,
    pub source_branch: String,
    pub target_branch: String,
    pub reviewers: Vec<String>,
    /// Priority as a number (1 = highest, N = lowest)
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    /// Timestamp when PR was merged (empty if not merged)
    pub merged_at: String,
    /// Timestamp when PR was closed (empty if not closed)
    pub closed_at: String,
    pub custom_fields: HashMap<String, String>,
    /// ISO timestamp when soft-deleted (None if not deleted)
    pub deleted_at: Option<String>,
}

/// Options for updating a PR
#[derive(Debug, Clone, Default)]
pub struct UpdatePrOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
    pub reviewers: Option<Vec<String>>,
    /// Priority as a number (1 = highest). None = don't update.
    pub priority: Option<u32>,
    pub custom_fields: HashMap<String, String>,
}

/// Result of PR update
#[derive(Debug, Clone)]
pub struct UpdatePrResult {
    pub pr: PullRequest,
    pub manifest: CentyManifest,
}

/// Result of PR deletion
#[derive(Debug, Clone)]
pub struct DeletePrResult {
    pub manifest: CentyManifest,
}

/// Result of soft-deleting a PR
#[derive(Debug, Clone)]
pub struct SoftDeletePrResult {
    pub pr: PullRequest,
    pub manifest: CentyManifest,
}

/// Result of restoring a soft-deleted PR
#[derive(Debug, Clone)]
pub struct RestorePrResult {
    pub pr: PullRequest,
    pub manifest: CentyManifest,
}

/// A PR with its source project information
#[derive(Debug, Clone)]
pub struct PrWithProject {
    pub pr: PullRequest,
    pub project_path: String,
    pub project_name: String,
}

/// Result of searching for PRs by UUID across projects
#[derive(Debug, Clone)]
pub struct GetPrsByUuidResult {
    pub prs: Vec<PrWithProject>,
    pub errors: Vec<String>,
}

/// Get a single PR by its ID (UUID)
pub async fn get_pr(
    project_path: &Path,
    pr_id: &str,
) -> Result<PullRequest, PrCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(PrCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let pr_path = centy_path.join("prs").join(pr_id);

    if !pr_path.exists() {
        return Err(PrCrudError::PrNotFound(pr_id.to_string()));
    }

    read_pr_from_disk(&pr_path, pr_id).await
}

/// List all PRs with optional filtering
pub async fn list_prs(
    project_path: &Path,
    status_filter: Option<&str>,
    source_branch_filter: Option<&str>,
    target_branch_filter: Option<&str>,
    priority_filter: Option<u32>,
    include_deleted: bool,
) -> Result<Vec<PullRequest>, PrCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(PrCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let prs_path = centy_path.join("prs");

    if !prs_path.exists() {
        return Ok(Vec::new());
    }

    // Reconcile display numbers to resolve any conflicts from concurrent creation
    reconcile_pr_display_numbers(&prs_path).await?;

    let mut prs = Vec::new();
    let mut entries = fs::read_dir(&prs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(folder_name) = entry.file_name().to_str() {
                if !is_valid_pr_folder(folder_name) {
                    continue;
                }

                match read_pr_from_disk(&entry.path(), folder_name).await {
                    Ok(pr) => {
                        // Apply filters
                        let status_match = status_filter
                            .is_none_or(|s| pr.metadata.status == s);
                        let source_match = source_branch_filter
                            .is_none_or(|s| pr.metadata.source_branch == s);
                        let target_match = target_branch_filter
                            .is_none_or(|t| pr.metadata.target_branch == t);
                        let priority_match = priority_filter
                            .is_none_or(|p| pr.metadata.priority == p);
                        let deleted_match = include_deleted || pr.metadata.deleted_at.is_none();

                        if status_match && source_match && target_match && priority_match && deleted_match {
                            prs.push(pr);
                        }
                    }
                    Err(_) => {
                        // Skip PRs that can't be read
                    }
                }
            }
        }
    }

    // Sort by display number (human-readable ordering)
    prs.sort_by_key(|p| p.metadata.display_number);

    Ok(prs)
}

/// Get a PR by its display number (human-readable number like 1, 2, 3)
pub async fn get_pr_by_display_number(
    project_path: &Path,
    display_number: u32,
) -> Result<PullRequest, PrCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(PrCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let prs_path = centy_path.join("prs");

    if !prs_path.exists() {
        return Err(PrCrudError::PrDisplayNumberNotFound(display_number));
    }

    // Reconcile first to ensure display numbers are unique
    reconcile_pr_display_numbers(&prs_path).await?;

    let mut entries = fs::read_dir(&prs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(folder_name) = entry.file_name().to_str() {
                if !is_valid_pr_folder(folder_name) {
                    continue;
                }

                let metadata_path = entry.path().join("metadata.json");
                if !metadata_path.exists() {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&metadata_path).await {
                    if let Ok(metadata) = serde_json::from_str::<PrMetadata>(&content) {
                        if metadata.common.display_number == display_number {
                            return read_pr_from_disk(&entry.path(), folder_name).await;
                        }
                    }
                }
            }
        }
    }

    Err(PrCrudError::PrDisplayNumberNotFound(display_number))
}

/// Search for PRs by UUID across all tracked projects
/// This is a global search that doesn't require a project_path
pub async fn get_prs_by_uuid(
    uuid: &str,
    projects: &[ProjectInfo],
) -> Result<GetPrsByUuidResult, PrCrudError> {
    // Validate that uuid is a valid UUID format
    if !is_uuid(uuid) {
        return Err(PrCrudError::InvalidPrFormat(
            "Only UUID format is supported for global search".to_string(),
        ));
    }

    let mut found_prs = Vec::new();
    let mut errors = Vec::new();

    for project in projects {
        // Skip uninitialized projects
        if !project.initialized {
            continue;
        }

        let project_path = Path::new(&project.path);

        // Try to get the PR from this project
        match get_pr(project_path, uuid).await {
            Ok(pr) => {
                let project_name = project.name.clone().unwrap_or_else(|| {
                    project_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| project.path.clone())
                });

                found_prs.push(PrWithProject {
                    pr,
                    project_path: project.path.clone(),
                    project_name,
                });
            }
            Err(PrCrudError::PrNotFound(_)) => {
                // Not an error - PR simply doesn't exist in this project
            }
            Err(PrCrudError::NotInitialized) => {
                // Skip - project not properly initialized
            }
            Err(e) => {
                // Log non-fatal errors but continue searching
                errors.push(format!("Error searching {}: {}", project.path, e));
            }
        }
    }

    Ok(GetPrsByUuidResult {
        prs: found_prs,
        errors,
    })
}

/// Update an existing PR
pub async fn update_pr(
    project_path: &Path,
    pr_id: &str,
    options: UpdatePrOptions,
) -> Result<UpdatePrResult, PrCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(PrCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let pr_path = centy_path.join("prs").join(pr_id);

    if !pr_path.exists() {
        return Err(PrCrudError::PrNotFound(pr_id.to_string()));
    }

    // Read config for priority_levels validation
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    // Read current PR
    let current = read_pr_from_disk(&pr_path, pr_id).await?;

    // Apply updates
    let new_title = options.title.unwrap_or(current.title);
    let new_description = options.description.unwrap_or(current.description);
    let new_status = options.status.unwrap_or(current.metadata.status);
    let new_source_branch = options.source_branch.unwrap_or(current.metadata.source_branch);
    let new_target_branch = options.target_branch.unwrap_or(current.metadata.target_branch);
    let new_reviewers = options.reviewers.unwrap_or(current.metadata.reviewers);

    // Get allowed PR statuses from config or use defaults
    let allowed_statuses = default_pr_statuses();

    // Lenient validation: log warning if status is not in allowed_states
    validate_pr_status(&new_status, &allowed_statuses);

    // Handle status transitions that set merged_at or closed_at
    let mut new_merged_at = current.metadata.merged_at.clone();
    let mut new_closed_at = current.metadata.closed_at.clone();

    if new_status == "merged" && new_merged_at.is_empty() {
        new_merged_at = now_iso();
    }
    if new_status == "closed" && new_closed_at.is_empty() {
        new_closed_at = now_iso();
    }

    // Validate and apply priority update
    let new_priority = match options.priority {
        Some(p) => {
            validate_priority(p, priority_levels)?;
            p
        }
        None => current.metadata.priority,
    };

    // Merge custom fields
    let mut new_custom_fields = current.metadata.custom_fields;
    for (key, value) in options.custom_fields {
        new_custom_fields.insert(key, value);
    }

    // Create updated metadata
    let updated_metadata = PrMetadata {
        common: crate::common::CommonMetadata {
            display_number: current.metadata.display_number,
            status: new_status.clone(),
            priority: new_priority,
            created_at: current.metadata.created_at.clone(),
            updated_at: now_iso(),
            custom_fields: new_custom_fields
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        },
        source_branch: new_source_branch.clone(),
        target_branch: new_target_branch.clone(),
        reviewers: new_reviewers.clone(),
        merged_at: new_merged_at.clone(),
        closed_at: new_closed_at.clone(),
        deleted_at: current.metadata.deleted_at.clone(),
    };

    // Generate updated content
    let pr_md = generate_pr_md(&new_title, &new_description);

    // Write files
    let pr_md_path = pr_path.join("pr.md");
    let metadata_path = pr_path.join("metadata.json");

    fs::write(&pr_md_path, &pr_md).await?;
    fs::write(&metadata_path, serde_json::to_string_pretty(&updated_metadata)?).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let pr = PullRequest {
        id: pr_id.to_string(),
        title: new_title,
        description: new_description,
        metadata: PrMetadataFlat {
            display_number: current.metadata.display_number,
            status: new_status,
            source_branch: new_source_branch,
            target_branch: new_target_branch,
            reviewers: new_reviewers,
            priority: new_priority,
            created_at: current.metadata.created_at,
            updated_at: updated_metadata.common.updated_at,
            merged_at: new_merged_at,
            closed_at: new_closed_at,
            custom_fields: new_custom_fields,
            deleted_at: current.metadata.deleted_at,
        },
    };

    Ok(UpdatePrResult { pr, manifest })
}

/// Delete a PR
pub async fn delete_pr(
    project_path: &Path,
    pr_id: &str,
) -> Result<DeletePrResult, PrCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(PrCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let pr_path = centy_path.join("prs").join(pr_id);

    if !pr_path.exists() {
        return Err(PrCrudError::PrNotFound(pr_id.to_string()));
    }

    // Remove the PR directory
    fs::remove_dir_all(&pr_path).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(DeletePrResult { manifest })
}

/// Soft-delete a PR (set deleted_at timestamp)
pub async fn soft_delete_pr(
    project_path: &Path,
    pr_id: &str,
) -> Result<SoftDeletePrResult, PrCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(PrCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let pr_path = centy_path.join("prs").join(pr_id);

    if !pr_path.exists() {
        return Err(PrCrudError::PrNotFound(pr_id.to_string()));
    }

    // Read current metadata
    let metadata_path = pr_path.join("metadata.json");
    let metadata_content = fs::read_to_string(&metadata_path).await?;
    let mut metadata: PrMetadata = serde_json::from_str(&metadata_content)?;

    // Check if already deleted
    if metadata.deleted_at.is_some() {
        return Err(PrCrudError::PrAlreadyDeleted(pr_id.to_string()));
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

    // Read and return the updated PR
    let pr = read_pr_from_disk(&pr_path, pr_id).await?;

    Ok(SoftDeletePrResult { pr, manifest })
}

/// Restore a soft-deleted PR (clear deleted_at timestamp)
pub async fn restore_pr(
    project_path: &Path,
    pr_id: &str,
) -> Result<RestorePrResult, PrCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(PrCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let pr_path = centy_path.join("prs").join(pr_id);

    if !pr_path.exists() {
        return Err(PrCrudError::PrNotFound(pr_id.to_string()));
    }

    // Read current metadata
    let metadata_path = pr_path.join("metadata.json");
    let metadata_content = fs::read_to_string(&metadata_path).await?;
    let mut metadata: PrMetadata = serde_json::from_str(&metadata_content)?;

    // Check if actually deleted
    if metadata.deleted_at.is_none() {
        return Err(PrCrudError::PrNotDeleted(pr_id.to_string()));
    }

    // Clear deleted_at and update updated_at
    metadata.deleted_at = None;
    metadata.common.updated_at = now_iso();

    // Write updated metadata
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    // Read and return the restored PR
    let pr = read_pr_from_disk(&pr_path, pr_id).await?;

    Ok(RestorePrResult { pr, manifest })
}

/// Read a PR from disk
async fn read_pr_from_disk(pr_path: &Path, pr_id: &str) -> Result<PullRequest, PrCrudError> {
    let pr_md_path = pr_path.join("pr.md");
    let metadata_path = pr_path.join("metadata.json");

    if !pr_md_path.exists() || !metadata_path.exists() {
        return Err(PrCrudError::InvalidPrFormat(format!(
            "PR {pr_id} is missing required files"
        )));
    }

    // Read pr.md
    let pr_md = fs::read_to_string(&pr_md_path).await?;
    let (title, description) = parse_pr_md(&pr_md);

    // Read metadata
    let metadata_content = fs::read_to_string(&metadata_path).await?;
    let metadata: PrMetadata = serde_json::from_str(&metadata_content)?;

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

    Ok(PullRequest {
        id: pr_id.to_string(),
        title,
        description,
        metadata: PrMetadataFlat {
            display_number: metadata.common.display_number,
            status: metadata.common.status,
            source_branch: metadata.source_branch,
            target_branch: metadata.target_branch,
            reviewers: metadata.reviewers,
            priority: metadata.common.priority,
            created_at: metadata.common.created_at,
            updated_at: metadata.common.updated_at,
            merged_at: metadata.merged_at,
            closed_at: metadata.closed_at,
            custom_fields,
            deleted_at: metadata.deleted_at,
        },
    })
}

/// Parse pr.md content to extract title and description
fn parse_pr_md(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return (String::new(), String::new());
    }

    // First line should be the title (# Title)
    let title = lines[0]
        .strip_prefix('#')
        .map_or(lines[0], str::trim)
        .to_string();

    // Rest is description (skip empty lines after title)
    let description_lines: Vec<&str> = lines[1..]
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect();

    let description = description_lines.join("\n").trim_end().to_string();

    (title, description)
}

/// Generate the PR markdown content
fn generate_pr_md(title: &str, description: &str) -> String {
    if description.is_empty() {
        format!("# {title}\n")
    } else {
        format!("# {title}\n\n{description}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pr_md_with_description() {
        let content = "# My PR Title\n\nThis is the description.\nWith multiple lines.";
        let (title, description) = parse_pr_md(content);
        assert_eq!(title, "My PR Title");
        assert_eq!(description, "This is the description.\nWith multiple lines.");
    }

    #[test]
    fn test_parse_pr_md_title_only() {
        let content = "# My PR Title\n";
        let (title, description) = parse_pr_md(content);
        assert_eq!(title, "My PR Title");
        assert_eq!(description, "");
    }

    #[test]
    fn test_parse_pr_md_empty() {
        let content = "";
        let (title, description) = parse_pr_md(content);
        assert_eq!(title, "");
        assert_eq!(description, "");
    }
}
