use crate::config::read_config;
use crate::manifest::{
    read_manifest, write_manifest, update_manifest_timestamp, CentyManifest,
};
use crate::utils::get_centy_path;
use crate::issue::priority::{default_priority, validate_priority, PriorityError};
use super::git::{detect_current_branch, get_default_branch, is_git_repository, validate_branch_exists, GitError};
use super::id::generate_pr_id;
use super::metadata::PrMetadata;
use super::reconcile::{get_next_pr_display_number, ReconcileError};
use super::status::{default_pr_statuses, validate_pr_status};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;
use tracing::warn;

#[derive(Error, Debug)]
pub enum PrError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Title is required")]
    TitleRequired,

    #[error("Source branch is required")]
    SourceBranchRequired,

    #[error("Invalid priority: {0}")]
    InvalidPriority(#[from] PriorityError),

    #[error("Git error: {0}")]
    GitError(#[from] GitError),

    #[error("Reconcile error: {0}")]
    ReconcileError(#[from] ReconcileError),

    #[error("Not a git repository")]
    NotGitRepository,

    #[error("Source branch '{0}' does not exist")]
    SourceBranchNotFound(String),

    #[error("Target branch '{0}' does not exist")]
    TargetBranchNotFound(String),
}

/// Options for creating a PR
#[derive(Debug, Clone, Default)]
pub struct CreatePrOptions {
    pub title: String,
    pub description: String,
    /// Source branch name. If empty, will try to detect current branch.
    pub source_branch: Option<String>,
    /// Target branch name. If empty, defaults to "main" or "master".
    pub target_branch: Option<String>,
    /// Linked issue IDs or display numbers
    pub linked_issues: Vec<String>,
    /// Reviewer usernames/identifiers
    pub reviewers: Vec<String>,
    /// Priority as a number (1 = highest). None = use default.
    pub priority: Option<u32>,
    /// Initial status. Defaults to "draft".
    pub status: Option<String>,
    pub custom_fields: HashMap<String, String>,
    /// Optional template name (without .md extension)
    pub template: Option<String>,
}

/// Result of PR creation
#[derive(Debug, Clone)]
pub struct CreatePrResult {
    /// UUID-based PR ID (folder name)
    pub id: String,
    /// Human-readable display number (1, 2, 3...)
    pub display_number: u32,
    pub created_files: Vec<String>,
    pub manifest: CentyManifest,
    /// The source branch that was used (may be auto-detected)
    pub detected_source_branch: String,
}

/// Create a new PR
pub async fn create_pr(
    project_path: &Path,
    options: CreatePrOptions,
) -> Result<CreatePrResult, PrError> {
    // Validate title
    if options.title.trim().is_empty() {
        return Err(PrError::TitleRequired);
    }

    // Check if centy is initialized
    let manifest = read_manifest(project_path)
        .await?
        .ok_or(PrError::NotInitialized)?;

    // Check if this is a git repository
    if !is_git_repository(project_path) {
        warn!("Creating PR in a non-git directory. Branch validation will be skipped.");
    }

    let centy_path = get_centy_path(project_path);
    let prs_path = centy_path.join("prs");

    // Ensure prs directory exists
    if !prs_path.exists() {
        fs::create_dir_all(&prs_path).await?;
    }

    // Generate UUID for folder name (prevents git conflicts)
    let pr_id = generate_pr_id();

    // Get next display number for human-readable reference
    let display_number = get_next_pr_display_number(&prs_path).await?;

    // Determine source branch
    let source_branch = match options.source_branch {
        Some(branch) if !branch.is_empty() => {
            // Validate branch exists if in git repo
            if is_git_repository(project_path) {
                if !validate_branch_exists(project_path, &branch).unwrap_or(true) {
                    warn!(branch = %branch, "Source branch does not exist. Creating PR anyway.");
                }
            }
            branch
        }
        _ => {
            // Try to auto-detect current branch
            if is_git_repository(project_path) {
                detect_current_branch(project_path)?
            } else {
                return Err(PrError::SourceBranchRequired);
            }
        }
    };

    // Determine target branch
    let target_branch = match options.target_branch {
        Some(branch) if !branch.is_empty() => {
            // Validate branch exists if in git repo
            if is_git_repository(project_path) {
                if !validate_branch_exists(project_path, &branch).unwrap_or(true) {
                    warn!(branch = %branch, "Target branch does not exist. Creating PR anyway.");
                }
            }
            branch
        }
        _ => {
            // Use default branch (main or master)
            if is_git_repository(project_path) {
                get_default_branch(project_path)
            } else {
                "main".to_string()
            }
        }
    };

    // Read config for defaults and priority_levels
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map(|c| c.priority_levels).unwrap_or(3);

    // Determine priority
    let priority = match options.priority {
        Some(p) => {
            validate_priority(p, priority_levels)?;
            p
        }
        None => {
            // Try config defaults first, then use calculated default
            config
                .as_ref()
                .and_then(|c| c.defaults.get("priority"))
                .and_then(|p| p.parse::<u32>().ok())
                .unwrap_or_else(|| default_priority(priority_levels))
        }
    };

    // Determine status - use provided value or default to "draft"
    let status = options.status.unwrap_or_else(|| "draft".to_string());

    // Get allowed PR statuses from config or use defaults
    let allowed_statuses = default_pr_statuses();

    // Lenient validation: log warning if status is not in allowed_states
    validate_pr_status(&status, &allowed_statuses);

    // Build custom fields with defaults from config
    let mut custom_field_values: HashMap<String, serde_json::Value> = HashMap::new();

    if let Some(ref config) = config {
        // Apply defaults from config
        for field in &config.custom_fields {
            if let Some(default_value) = &field.default_value {
                custom_field_values.insert(
                    field.name.clone(),
                    serde_json::Value::String(default_value.clone()),
                );
            }
        }
    }

    // Override with provided custom fields
    for (key, value) in &options.custom_fields {
        custom_field_values.insert(key.clone(), serde_json::Value::String(value.clone()));
    }

    // Create metadata
    let metadata = PrMetadata::new(
        display_number,
        status,
        source_branch.clone(),
        target_branch,
        options.linked_issues,
        options.reviewers,
        priority,
        custom_field_values,
    );

    // Create PR content
    let pr_md = generate_pr_md(&options.title, &options.description);

    // Write files (using UUID as folder name)
    let pr_folder = prs_path.join(&pr_id);
    fs::create_dir_all(&pr_folder).await?;

    let pr_md_path = pr_folder.join("pr.md");
    let metadata_path = pr_folder.join("metadata.json");
    let assets_path = pr_folder.join("assets");

    fs::write(&pr_md_path, &pr_md).await?;
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?).await?;
    fs::create_dir_all(&assets_path).await?;

    // Update manifest timestamp
    let mut manifest = manifest;
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let created_files = vec![
        format!(".centy/prs/{}/pr.md", pr_id),
        format!(".centy/prs/{}/metadata.json", pr_id),
        format!(".centy/prs/{}/assets/", pr_id),
    ];

    Ok(CreatePrResult {
        id: pr_id,
        display_number,
        created_files,
        manifest,
        detected_source_branch: source_branch,
    })
}

/// Generate the PR markdown content
fn generate_pr_md(title: &str, description: &str) -> String {
    if description.is_empty() {
        format!("# {}\n", title)
    } else {
        format!("# {}\n\n{}\n", title, description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pr_md_with_description() {
        let md = generate_pr_md("Add feature", "This PR adds a new feature.");
        assert_eq!(md, "# Add feature\n\nThis PR adds a new feature.\n");
    }

    #[test]
    fn test_generate_pr_md_without_description() {
        let md = generate_pr_md("Fix bug", "");
        assert_eq!(md, "# Fix bug\n");
    }
}
