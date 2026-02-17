use super::crud::{Issue, IssueMetadataFlat};
use super::id::generate_issue_id;
use super::metadata::{IssueFrontmatter, IssueMetadata};
use super::org_registry::{get_next_org_display_number, OrgIssueRegistryError};
use super::planning::{add_planning_note, is_planning_status};
use super::priority::{default_priority, priority_label, validate_priority, PriorityError};
use super::reconcile::{get_next_display_number, ReconcileError};
use super::status::{validate_status_for_project, StatusError};
use crate::common::{generate_frontmatter, sync_to_org_projects, OrgSyncResult};
use crate::config::item_type_config::read_item_type_config;
use crate::config::read_config;
use crate::manifest::{read_manifest, update_manifest, write_manifest, CentyManifest};
use crate::registry::get_project_info;
use crate::template::{IssueTemplateContext, TemplateEngine, TemplateError};
use crate::utils::{format_markdown, get_centy_path, now_iso};
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

    #[error("Cannot create org issue: project has no organization")]
    NoOrganization,

    #[error("Org registry error: {0}")]
    OrgRegistryError(#[from] OrgIssueRegistryError),

    #[error("Registry error: {0}")]
    RegistryError(String),
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
    /// Create as organization-wide issue (syncs to all org projects)
    pub is_org_issue: bool,
}

/// Result of issue creation
#[derive(Debug, Clone)]
pub struct CreateIssueResult {
    /// UUID-based issue ID (folder name)
    pub id: String,
    /// Human-readable display number (1, 2, 3...)
    pub display_number: u32,
    /// Org-level display number (only for org issues)
    pub org_display_number: Option<u32>,
    /// Legacy field for backward compatibility (same as id)
    #[deprecated(note = "Use `id` instead")]
    pub issue_number: String,
    pub created_files: Vec<String>,
    pub manifest: CentyManifest,
    /// Results from syncing to other org projects (empty for non-org issues)
    pub sync_results: Vec<OrgSyncResult>,
}

/// Resolve organization info for an org issue.
async fn resolve_org_info(
    project_path: &Path,
    is_org_issue: bool,
) -> Result<(Option<String>, Option<u32>), IssueError> {
    if !is_org_issue {
        return Ok((None, None));
    }

    let project_path_str = project_path.to_string_lossy().to_string();
    let project_info = get_project_info(&project_path_str)
        .await
        .map_err(|e| IssueError::RegistryError(e.to_string()))?;

    match project_info.and_then(|p| p.organization_slug) {
        Some(slug) => {
            let org_num = get_next_org_display_number(&slug).await?;
            Ok((Some(slug), Some(org_num)))
        }
        None => Err(IssueError::NoOrganization),
    }
}

/// Resolve priority from options or config defaults.
fn resolve_priority(
    priority_opt: Option<u32>,
    config: Option<&crate::config::CentyConfig>,
    priority_levels: u32,
) -> Result<u32, IssueError> {
    match priority_opt {
        Some(p) => {
            validate_priority(p, priority_levels)?;
            Ok(p)
        }
        None => Ok(config
            .and_then(|c| c.defaults.get("priority"))
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or_else(|| default_priority(priority_levels))),
    }
}

/// Build custom fields from config defaults and provided values.
fn build_custom_fields(
    config: Option<&crate::config::CentyConfig>,
    provided_fields: &HashMap<String, String>,
) -> HashMap<String, serde_json::Value> {
    let mut fields: HashMap<String, serde_json::Value> = HashMap::new();

    // Apply defaults from config
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

    // Override with provided values
    for (key, value) in provided_fields {
        fields.insert(key.clone(), serde_json::Value::String(value.clone()));
    }

    fields
}

/// Build Issue struct for org sync from creation data.
fn build_issue_for_sync(
    issue_id: &str,
    options: &CreateIssueOptions,
    display_number: u32,
    metadata: &IssueMetadata,
) -> Issue {
    #[allow(deprecated)]
    Issue {
        id: issue_id.to_string(),
        issue_number: issue_id.to_string(),
        title: options.title.clone(),
        description: options.description.clone(),
        metadata: IssueMetadataFlat {
            display_number,
            status: metadata.common.status.clone(),
            priority: metadata.common.priority,
            created_at: metadata.common.created_at.clone(),
            updated_at: metadata.common.updated_at.clone(),
            custom_fields: options.custom_fields.clone(),
            draft: metadata.draft,
            deleted_at: metadata.deleted_at.clone(),
            is_org_issue: metadata.is_org_issue,
            org_slug: metadata.org_slug.clone(),
            org_display_number: metadata.org_display_number,
        },
    }
}

/// Create a new issue
pub async fn create_issue(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    if options.title.trim().is_empty() {
        return Err(IssueError::TitleRequired);
    }

    let manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    fs::create_dir_all(&issues_path).await?;

    let (org_slug, org_display_number) =
        resolve_org_info(project_path, options.is_org_issue).await?;
    let issue_id = generate_issue_id();
    let display_number = get_next_display_number(&issues_path).await?;

    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);
    let priority = resolve_priority(options.priority, config.as_ref(), priority_levels)?;

    let item_type_config = read_item_type_config(project_path, "issues")
        .await
        .ok()
        .flatten();
    let status = options.status.clone().unwrap_or_else(|| {
        item_type_config
            .as_ref()
            .and_then(|c| c.default_status.clone())
            .unwrap_or_else(|| "open".to_string())
    });
    validate_status_for_project(project_path, "issues", &status).await?;

    let custom_field_values = build_custom_fields(config.as_ref(), &options.custom_fields);
    let draft = options.draft.unwrap_or(false);
    let now = now_iso();

    // Build frontmatter
    let frontmatter = IssueFrontmatter {
        display_number,
        status: status.clone(),
        priority,
        created_at: now.clone(),
        updated_at: now,
        draft,
        deleted_at: None,
        is_org_issue: org_slug.is_some(),
        org_slug: org_slug.clone(),
        org_display_number,
        custom_fields: options.custom_fields.clone(),
    };

    // Generate content with optional template
    let (display_title, description) = if let Some(ref template_name) = options.template {
        let template_engine = TemplateEngine::new();
        let context = IssueTemplateContext {
            title: options.title.clone(),
            description: options.description.clone(),
            priority,
            priority_label: priority_label(priority, priority_levels),
            status: status.clone(),
            created_at: frontmatter.created_at.clone(),
            custom_fields: options.custom_fields.clone(),
        };
        // Template returns the full markdown, we need to extract title and description
        let templated = template_engine
            .render_issue(project_path, template_name, &context)
            .await?;
        // Extract title and description from templated content
        let (extracted_title, desc) = parse_templated_content(&templated);
        // Use extracted title if present, otherwise fall back to options.title
        let title = if extracted_title.is_empty() {
            options.title.clone()
        } else {
            extracted_title
        };
        (title, desc)
    } else {
        (options.title.clone(), options.description.clone())
    };

    // Handle planning note - we add it to the body, not the frontmatter
    let body = if is_planning_status(&status) {
        add_planning_note(&description)
    } else {
        description
    };

    // Generate full content with frontmatter
    let issue_content = generate_frontmatter(&frontmatter, &display_title, &body);

    // Write single .md file
    let issue_file = issues_path.join(format!("{issue_id}.md"));
    fs::write(&issue_file, format_markdown(&issue_content)).await?;

    let mut manifest = manifest;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let created_files = vec![format!(".centy/issues/{issue_id}.md")];

    // Build legacy metadata for org sync
    let metadata = if let Some(ref org) = org_slug {
        IssueMetadata::new_org_issue(
            display_number,
            org_display_number.unwrap_or(0),
            status.clone(),
            priority,
            org,
            custom_field_values,
            draft,
        )
    } else {
        IssueMetadata::new_draft(display_number, status, priority, custom_field_values, draft)
    };

    let sync_results = if options.is_org_issue {
        let issue = build_issue_for_sync(&issue_id, &options, display_number, &metadata);
        sync_to_org_projects(&issue, project_path).await
    } else {
        Vec::new()
    };

    #[allow(deprecated)]
    Ok(CreateIssueResult {
        id: issue_id.clone(),
        display_number,
        org_display_number,
        issue_number: issue_id,
        created_files,
        manifest,
        sync_results,
    })
}

/// Parse templated content to extract just the description (body without title)
fn parse_templated_content(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (String::new(), String::new());
    }

    // Find title line
    let mut title_idx = 0;
    for (idx, line) in lines.iter().enumerate() {
        if line.starts_with('#') {
            title_idx = idx;
            break;
        }
    }

    let title = lines
        .get(title_idx)
        .map(|line| line.strip_prefix('#').map_or(*line, str::trim))
        .unwrap_or("")
        .to_string();

    let description = lines
        .get(title_idx.saturating_add(1)..)
        .unwrap_or(&[])
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();

    (title, description)
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

    Ok(format!("{:04}", max_number.saturating_add(1)))
}

/// Create a new issue
///
/// This is a thin wrapper around `create_issue` for backward compatibility.
/// Title is required.
///
/// # Errors
///
/// Returns an error if:
/// - Title is empty (`TitleRequired`)
/// - Any of the regular issue creation errors occur
pub async fn create_issue_with_title_generation(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    create_issue(project_path, options).await
}
