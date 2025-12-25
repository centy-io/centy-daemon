use crate::config::read_config;
use crate::llm::{generate_title, TitleGenerationError};
use crate::manifest::{
    read_manifest, write_manifest, update_manifest_timestamp, CentyManifest,
};
use crate::template::{IssueTemplateContext, TemplateEngine, TemplateError};
use crate::utils::{format_markdown, get_centy_path};
use super::id::generate_issue_id;
use super::metadata::IssueMetadata;
use super::priority::{default_priority, priority_label, validate_priority, PriorityError};
use super::reconcile::{get_next_display_number, ReconcileError};
use super::planning::{add_planning_note, is_planning_status};
use super::status::{validate_status, StatusError};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum IssueError {
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

    #[error("Invalid priority: {0}")]
    InvalidPriority(#[from] PriorityError),

    #[error("Invalid status: {0}")]
    InvalidStatus(#[from] StatusError),

    #[error("Template error: {0}")]
    TemplateError(#[from] TemplateError),

    #[error("Reconcile error: {0}")]
    ReconcileError(#[from] ReconcileError),

    #[error("Title generation failed: {0}")]
    TitleGenerationFailed(#[from] TitleGenerationError),
}

/// Options for creating an issue
#[derive(Debug, Clone, Default)]
pub struct CreateIssueOptions {
    pub title: String,
    pub description: String,
    /// Priority as a number (1 = highest). None = use default.
    pub priority: Option<u32>,
    pub status: Option<String>,
    pub custom_fields: HashMap<String, String>,
    /// Optional template name (without .md extension)
    pub template: Option<String>,
    /// Whether to create the issue as a draft
    pub draft: Option<bool>,
}

/// Result of issue creation
#[derive(Debug, Clone)]
pub struct CreateIssueResult {
    /// UUID-based issue ID (folder name)
    pub id: String,
    /// Human-readable display number (1, 2, 3...)
    pub display_number: u32,
    /// Legacy field for backward compatibility (same as id)
    #[deprecated(note = "Use `id` instead")]
    pub issue_number: String,
    pub created_files: Vec<String>,
    pub manifest: CentyManifest,
}

/// Create a new issue
#[allow(clippy::too_many_lines)]
pub async fn create_issue(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    // Validate title
    if options.title.trim().is_empty() {
        return Err(IssueError::TitleRequired);
    }

    // Check if centy is initialized
    let manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");

    // Ensure issues directory exists
    if !issues_path.exists() {
        fs::create_dir_all(&issues_path).await?;
    }

    // Generate UUID for folder name (prevents git conflicts)
    let issue_id = generate_issue_id();

    // Get next display number for human-readable reference
    let display_number = get_next_display_number(&issues_path).await?;

    // Read config for defaults and priority_levels
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

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

    // Determine status - use provided value, config.default_state, or fallback to "open"
    let status = options.status.unwrap_or_else(|| {
        config
            .as_ref().map_or_else(|| "open".to_string(), |c| c.default_state.clone())
    });

    // Strict validation: reject if status is not in allowed_states
    if let Some(ref config) = config {
        validate_status(&status, &config.allowed_states)?;
    }

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
    let metadata = IssueMetadata::new_draft(
        display_number,
        status.clone(),
        priority,
        custom_field_values,
        options.draft.unwrap_or(false),
    );

    // Create issue content
    let mut issue_md = if let Some(ref template_name) = options.template {
        // Use template engine
        let template_engine = TemplateEngine::new();
        let context = IssueTemplateContext {
            title: options.title.clone(),
            description: options.description.clone(),
            priority,
            priority_label: priority_label(priority, priority_levels),
            status,
            created_at: metadata.common.created_at.clone(),
            custom_fields: options.custom_fields.clone(),
        };
        template_engine
            .render_issue(project_path, template_name, &context)
            .await?
    } else {
        // Use default format
        generate_issue_md(&options.title, &options.description)
    };

    // Add planning note if status is "planning"
    if is_planning_status(&metadata.common.status) {
        issue_md = add_planning_note(&issue_md);
    }

    // Write files (using UUID as folder name)
    let issue_folder = issues_path.join(&issue_id);
    fs::create_dir_all(&issue_folder).await?;

    let issue_md_path = issue_folder.join("issue.md");
    let metadata_path = issue_folder.join("metadata.json");
    let assets_path = issue_folder.join("assets");

    fs::write(&issue_md_path, format_markdown(&issue_md)).await?;
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?).await?;
    fs::create_dir_all(&assets_path).await?;

    // Update manifest timestamp
    let mut manifest = manifest;
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let created_files = vec![
        format!(".centy/issues/{}/issue.md", issue_id),
        format!(".centy/issues/{}/metadata.json", issue_id),
        format!(".centy/issues/{}/assets/", issue_id),
    ];

    #[allow(deprecated)]
    Ok(CreateIssueResult {
        id: issue_id.clone(),
        display_number,
        issue_number: issue_id, // Legacy field
        created_files,
        manifest,
    })
}

/// Get the next issue number (zero-padded to 4 digits)
///
/// DEPRECATED: This function is kept for backward compatibility with legacy issues.
/// New issues use UUID folders with `display_number` in metadata.
/// Use `reconcile::get_next_display_number` for display numbers.
#[deprecated(note = "Use UUID-based folders with display_number in metadata")]
pub async fn get_next_issue_number(issues_path: &Path) -> Result<String, std::io::Error> {
    if !issues_path.exists() {
        return Ok("0001".to_string());
    }

    let mut max_number: u32 = 0;

    let mut entries = fs::read_dir(issues_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(num) = name.parse::<u32>() {
                    max_number = max_number.max(num);
                }
            }
        }
    }

    Ok(format!("{:04}", max_number + 1))
}

/// Generate the issue markdown content
fn generate_issue_md(title: &str, description: &str) -> String {
    if description.is_empty() {
        format!("# {title}\n")
    } else {
        format!("# {title}\n\n{description}\n")
    }
}

/// Create a new issue, optionally generating title via LLM
///
/// If title is empty and description is provided, attempts to generate
/// title using the configured LLM agent.
///
/// # Errors
///
/// Returns an error if:
/// - Both title and description are empty (`TitleRequired`)
/// - Title is empty and LLM is not configured (`TitleGenerationFailed`)
/// - Title is empty and LLM agent is not available (`TitleGenerationFailed`)
/// - LLM title generation fails for any other reason (`TitleGenerationFailed`)
/// - Any of the regular issue creation errors occur
pub async fn create_issue_with_title_generation(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    let title = if options.title.trim().is_empty() {
        // No title provided - try to generate one
        if options.description.trim().is_empty() {
            // Can't generate title without description
            return Err(IssueError::TitleRequired);
        }

        // Generate title via LLM
        let generated = generate_title(project_path, &options.description).await?;
        generated.title
    } else {
        options.title.clone()
    };

    // Create issue with (possibly generated) title
    let options_with_title = CreateIssueOptions {
        title,
        ..options
    };

    create_issue(project_path, options_with_title).await
}
