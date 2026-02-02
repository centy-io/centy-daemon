use super::git::{
    detect_current_branch, get_default_branch, is_git_repository, validate_branch_exists, GitError,
};
use super::id::generate_pr_id;
use super::metadata::PrFrontmatter;
use super::reconcile::{get_next_pr_display_number, ReconcileError};
use super::status::{default_pr_statuses, validate_pr_status};
use crate::common::generate_frontmatter;
use crate::config::read_config;
use crate::item::validation::priority::{default_priority, validate_priority, PriorityError};
use crate::manifest::{read_manifest, update_manifest, write_manifest, CentyManifest};
use crate::utils::{format_markdown, get_centy_path, now_iso};
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

/// Resolve source branch from options or auto-detect.
fn resolve_source_branch(
    project_path: &Path,
    source_opt: Option<String>,
) -> Result<String, PrError> {
    match source_opt {
        Some(branch) if !branch.is_empty() => {
            if is_git_repository(project_path)
                && !validate_branch_exists(project_path, &branch).unwrap_or(true)
            {
                warn!(branch = %branch, "Source branch does not exist. Creating PR anyway.");
            }
            Ok(branch)
        }
        _ if is_git_repository(project_path) => Ok(detect_current_branch(project_path)?),
        _ => Err(PrError::SourceBranchRequired),
    }
}

/// Resolve target branch from options or use default.
fn resolve_target_branch(project_path: &Path, target_opt: Option<String>) -> String {
    match target_opt {
        Some(branch) if !branch.is_empty() => {
            if is_git_repository(project_path)
                && !validate_branch_exists(project_path, &branch).unwrap_or(true)
            {
                warn!(branch = %branch, "Target branch does not exist. Creating PR anyway.");
            }
            branch
        }
        _ if is_git_repository(project_path) => get_default_branch(project_path),
        _ => "main".to_string(),
    }
}

/// Build custom fields from config defaults and provided values.
fn build_pr_custom_fields(
    config: Option<&crate::config::CentyConfig>,
    provided_fields: &HashMap<String, String>,
) -> HashMap<String, serde_json::Value> {
    let mut fields: HashMap<String, serde_json::Value> = HashMap::new();
    if let Some(config) = config {
        for field in &config.custom_fields {
            if let Some(default_value) = &field.default_value {
                fields.insert(
                    field.name.clone(),
                    serde_json::Value::String(default_value.clone()),
                );
            }
        }
    }
    for (key, value) in provided_fields {
        fields.insert(key.clone(), serde_json::Value::String(value.clone()));
    }
    fields
}

/// Create a new PR
pub async fn create_pr(
    project_path: &Path,
    options: CreatePrOptions,
) -> Result<CreatePrResult, PrError> {
    if options.title.trim().is_empty() {
        return Err(PrError::TitleRequired);
    }

    let manifest = read_manifest(project_path)
        .await?
        .ok_or(PrError::NotInitialized)?;

    if !is_git_repository(project_path) {
        warn!("Creating PR in a non-git directory. Branch validation will be skipped.");
    }

    let centy_path = get_centy_path(project_path);
    let prs_path = centy_path.join("prs");
    fs::create_dir_all(&prs_path).await?;

    let pr_id = generate_pr_id();
    let display_number = get_next_pr_display_number(&prs_path).await?;

    let source_branch = resolve_source_branch(project_path, options.source_branch)?;
    let target_branch = resolve_target_branch(project_path, options.target_branch);

    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);
    let priority = match options.priority {
        Some(p) => {
            validate_priority(p, priority_levels)?;
            p
        }
        None => config
            .as_ref()
            .and_then(|c| c.defaults.get("priority"))
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or_else(|| default_priority(priority_levels)),
    };

    let status = options.status.unwrap_or_else(|| "draft".to_string());
    validate_pr_status(&status, &default_pr_statuses());

    let custom_field_values = build_pr_custom_fields(config.as_ref(), &options.custom_fields);
    let now = now_iso();

    // Build frontmatter for new format
    let frontmatter = PrFrontmatter {
        display_number,
        status,
        source_branch: source_branch.clone(),
        target_branch,
        priority,
        created_at: now.clone(),
        updated_at: now,
        reviewers: options.reviewers,
        merged_at: None,
        closed_at: None,
        deleted_at: None,
        custom_fields: custom_field_values
            .into_iter()
            .map(|(k, v)| {
                let str_val = match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                (k, str_val)
            })
            .collect(),
    };

    // Generate full content with frontmatter
    let pr_content = generate_frontmatter(&frontmatter, &options.title, &options.description);

    // Write single .md file
    let pr_file = prs_path.join(format!("{pr_id}.md"));
    fs::write(&pr_file, format_markdown(&pr_content)).await?;

    let mut manifest = manifest;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(CreatePrResult {
        id: pr_id.clone(),
        display_number,
        created_files: vec![format!(".centy/prs/{pr_id}.md")],
        manifest,
        detected_source_branch: source_branch,
    })
}
