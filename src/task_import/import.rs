use std::path::{Path, PathBuf};

use crate::issue::crud::{list_issues, Issue};
use super::config::ProviderConfig;
use super::error::TaskImportError;
use super::mapper::{map_external_task_to_create, map_external_task_to_update};
use super::provider::{ExternalTask, TaskProvider};

/// Options for importing tasks
pub struct ImportOptions {
    pub project_path: PathBuf,
    pub provider_config: ProviderConfig,
    pub filter: Option<ImportFilter>,
}

/// Filters to apply when importing
pub struct ImportFilter {
    /// Only import tasks with these labels
    pub labels: Option<Vec<String>>,
    /// Only import tasks with these statuses
    pub status: Option<Vec<String>>,
    /// Maximum number of tasks to import (0 = unlimited)
    pub limit: Option<usize>,
}

/// Result of import operation
pub struct ImportResult {
    pub total_fetched: usize,
    pub created: Vec<String>,          // Issue UUIDs created
    pub updated: Vec<String>,          // Issue UUIDs updated
    pub skipped: Vec<String>,          // External IDs skipped
    pub errors: Vec<ImportError>,      // Non-fatal errors
}

/// Error for a single task import
#[derive(Debug, Clone)]
pub struct ImportError {
    pub external_id: String,
    pub error: String,
}

/// Result of processing a single task
enum ProcessResult {
    Created(String),
    Updated(String),
    Skipped,
}

/// Import tasks from a provider
///
/// This is the main orchestration function that:
/// 1. Fetches tasks from the external provider
/// 2. Applies filters
/// 3. For each task, checks if it already exists (by import_metadata)
/// 4. Creates new or updates existing Centy issues
/// 5. Returns detailed results
pub async fn import_tasks(
    provider: &dyn TaskProvider,
    options: ImportOptions,
) -> Result<ImportResult, TaskImportError> {
    let mut result = ImportResult {
        total_fetched: 0,
        created: Vec::new(),
        updated: Vec::new(),
        skipped: Vec::new(),
        errors: Vec::new(),
    };

    // 1. Fetch tasks from provider
    let tasks = provider
        .list_tasks(&options.provider_config.source_id)
        .await?;

    result.total_fetched = tasks.len();

    // 2. Apply filters
    let filtered_tasks = apply_filters(tasks, &options.filter);

    // 3. Process each task
    for task in filtered_tasks {
        match process_single_task(&task, &options).await {
            Ok(ProcessResult::Created(uuid)) => result.created.push(uuid),
            Ok(ProcessResult::Updated(uuid)) => result.updated.push(uuid),
            Ok(ProcessResult::Skipped) => result.skipped.push(task.external_id.clone()),
            Err(e) => result.errors.push(ImportError {
                external_id: task.external_id.clone(),
                error: e,
            }),
        }
    }

    Ok(result)
}

/// Process a single external task - create new or update existing issue
async fn process_single_task(
    task: &ExternalTask,
    options: &ImportOptions,
) -> Result<ProcessResult, String> {
    // Check for existing issue with same import_metadata
    let existing = find_existing_imported_issue(
        &options.project_path,
        &options.provider_config.provider,
        &options.provider_config.source_id,
        &task.external_id,
    )
    .await
    .map_err(|e| format!("Failed to check for existing issue: {e}"))?;

    if let Some(existing_issue) = existing {
        // Update existing issue
        update_imported_issue(&options.project_path, &existing_issue, task, &options.provider_config)
            .await
            .map_err(|e| format!("Failed to update issue: {e}"))?;
        Ok(ProcessResult::Updated(existing_issue.id))
    } else {
        // Create new issue
        let issue_id = create_imported_issue(&options.project_path, task, &options.provider_config)
            .await
            .map_err(|e| format!("Failed to create issue: {e}"))?;
        Ok(ProcessResult::Created(issue_id))
    }
}

/// Find existing Centy issue by import metadata
async fn find_existing_imported_issue(
    project_path: &Path,
    provider: &str,
    source_id: &str,
    external_id: &str,
) -> Result<Option<Issue>, String> {
    // List all issues (including deleted to avoid duplicates)
    let issues = list_issues(project_path, None, None, None, true)
        .await
        .map_err(|e| e.to_string())?;

    // Find issue with matching import_metadata
    Ok(issues.into_iter().find(|issue| {
        if let Some(import_meta) = &issue.metadata.import_metadata {
            import_meta.provider == provider
                && import_meta.source_id == source_id
                && import_meta.external_id == external_id
        } else {
            false
        }
    }))
}

/// Create a new Centy issue from external task
async fn create_imported_issue(
    project_path: &Path,
    task: &ExternalTask,
    config: &ProviderConfig,
) -> Result<String, String> {
    let (title, description, status, priority, custom_fields, import_metadata) =
        map_external_task_to_create(task, config).map_err(|e| e.to_string())?;

    // Use the issue CRUD create function
    use crate::issue::create::{create_issue, CreateIssueOptions};

    // Convert HashMap<String, String> to HashMap<String, String> (already compatible)
    let options = CreateIssueOptions {
        title,
        description,
        priority: Some(priority),
        status: Some(status),
        custom_fields,
        template: None,
        draft: None,
        is_org_issue: false,
        import_metadata: Some(import_metadata),
    };

    let result = create_issue(project_path, options)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.id)
}

/// Update an existing Centy issue from external task
async fn update_imported_issue(
    project_path: &Path,
    existing_issue: &Issue,
    task: &ExternalTask,
    config: &ProviderConfig,
) -> Result<(), String> {
    let existing_imported_at = existing_issue
        .metadata
        .import_metadata
        .as_ref()
        .map(|m| m.imported_at.clone())
        .unwrap_or_else(|| crate::utils::now_iso());

    let (title, description, status, custom_fields, import_metadata) =
        map_external_task_to_update(task, config, existing_imported_at)
            .map_err(|e| e.to_string())?;

    // Use the issue CRUD update function
    // Note: We'll need to update the issue CRUD to accept import_metadata
    use crate::issue::crud::{update_issue, UpdateIssueOptions};

    let options = UpdateIssueOptions {
        title: Some(title),
        description: Some(description),
        status: Some(status),
        priority: None, // Don't override priority on update
        custom_fields,
        draft: None,
        import_metadata: Some(import_metadata),
    };

    update_issue(project_path, &existing_issue.id, options)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Apply filters to tasks
fn apply_filters(tasks: Vec<ExternalTask>, filter: &Option<ImportFilter>) -> Vec<ExternalTask> {
    let Some(filter) = filter else {
        return tasks;
    };

    let mut filtered = tasks;

    // Filter by labels
    if let Some(ref label_filter) = filter.labels {
        filtered.retain(|task| task.labels.iter().any(|label| label_filter.contains(label)));
    }

    // Filter by status
    if let Some(ref status_filter) = filter.status {
        filtered.retain(|task| status_filter.contains(&task.status));
    }

    // Apply limit
    if let Some(limit) = filter.limit {
        if limit > 0 {
            filtered.truncate(limit);
        }
    }

    filtered
}
