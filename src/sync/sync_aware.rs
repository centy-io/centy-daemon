//! Sync-aware wrappers for CRUD operations.
//!
//! This module provides wrappers around the issue/doc/PR CRUD operations
//! that automatically handle sync operations:
//! - Before reads: Pull latest changes from remote
//! - After writes: Commit and push changes to remote
//!
//! All sync operations are best-effort and non-blocking. Failures are logged
//! but don't cause the underlying CRUD operation to fail.

use super::manager::CentySyncManager;
use crate::docs::{
    self, CreateDocOptions, CreateDocResult, DeleteDocResult, Doc, DocError,
    DuplicateDocOptions, DuplicateDocResult, MoveDocOptions, MoveDocResult,
    RestoreDocResult, SoftDeleteDocResult, UpdateDocOptions, UpdateDocResult,
};
use crate::issue::{
    self,
    create::{CreateIssueOptions, CreateIssueResult, IssueError},
    crud::{
        DeleteIssueResult, DuplicateIssueOptions, DuplicateIssueResult, Issue, IssueCrudError,
        MoveIssueOptions, MoveIssueResult, RestoreIssueResult, SoftDeleteIssueResult,
        UpdateIssueOptions, UpdateIssueResult,
    },
};
use crate::pr::{
    self, CreatePrOptions, CreatePrResult, DeletePrResult, PullRequest, UpdatePrOptions,
    UpdatePrResult, RestorePrResult, SoftDeletePrResult,
};
use crate::pr::crud::PrCrudError;
use crate::pr::create::PrError;
use std::path::Path;
use tracing::{debug, warn};

/// Get a sync manager for the project, or None if sync is disabled/unavailable.
async fn get_sync_manager(project_path: &Path) -> Option<CentySyncManager> {
    match CentySyncManager::new(project_path).await {
        Ok(manager) if manager.is_enabled() => Some(manager),
        Ok(_) => None,
        Err(e) => {
            debug!("Sync not available: {}", e);
            None
        }
    }
}

/// Pull latest changes before a read operation.
/// Returns true if sync was performed, false otherwise.
async fn sync_before_read(project_path: &Path) -> bool {
    if let Some(manager) = get_sync_manager(project_path).await {
        if let Err(e) = manager.pull_before_read().await {
            warn!("Failed to pull before read: {}", e);
        }
        // Sync from worktree to project
        if let Err(e) = manager.sync_to_project(Path::new(".centy")).await {
            warn!("Failed to sync to project: {}", e);
        }
        true
    } else {
        false
    }
}

/// Commit and push after a write operation.
async fn sync_after_write(project_path: &Path, message: &str) {
    if let Some(manager) = get_sync_manager(project_path).await {
        // First sync from project to worktree
        if let Err(e) = manager.sync_from_project(Path::new(".centy")).await {
            warn!("Failed to sync from project: {}", e);
            return;
        }
        // Then commit and push
        if let Err(e) = manager.commit_and_push(message).await {
            warn!("Failed to commit and push: {}", e);
        }
    }
}

// =============================================================================
// Issue CRUD wrappers
// =============================================================================

/// Get a single issue by its ID (sync-aware).
///
/// Pulls latest changes before reading.
pub async fn get_issue(project_path: &Path, issue_id: &str) -> Result<Issue, IssueCrudError> {
    sync_before_read(project_path).await;
    issue::get_issue(project_path, issue_id).await
}

/// List all issues with optional filtering (sync-aware).
///
/// Pulls latest changes before reading.
pub async fn list_issues(
    project_path: &Path,
    status_filter: Option<&str>,
    priority_filter: Option<u32>,
    draft_filter: Option<bool>,
    include_deleted: bool,
) -> Result<Vec<Issue>, IssueCrudError> {
    sync_before_read(project_path).await;
    issue::list_issues(
        project_path,
        status_filter,
        priority_filter,
        draft_filter,
        include_deleted,
    )
    .await
}

/// Get an issue by its display number (sync-aware).
///
/// Pulls latest changes before reading.
pub async fn get_issue_by_display_number(
    project_path: &Path,
    display_number: u32,
) -> Result<Issue, IssueCrudError> {
    sync_before_read(project_path).await;
    issue::get_issue_by_display_number(project_path, display_number).await
}

/// Create a new issue (sync-aware).
///
/// Commits and pushes after creation.
pub async fn create_issue(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    // Pull before creating to get latest display numbers
    sync_before_read(project_path).await;

    let result = issue::create_issue(project_path, options).await?;

    sync_after_write(
        project_path,
        &format!("centy: Create issue #{}", result.display_number),
    )
    .await;

    Ok(result)
}

/// Create a new issue with optional title generation (sync-aware).
///
/// Commits and pushes after creation.
pub async fn create_issue_with_title_generation(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    // Pull before creating to get latest display numbers
    sync_before_read(project_path).await;

    let result = issue::create_issue_with_title_generation(project_path, options).await?;

    sync_after_write(
        project_path,
        &format!("centy: Create issue #{}", result.display_number),
    )
    .await;

    Ok(result)
}

/// Update an existing issue (sync-aware).
///
/// Pulls before reading, commits and pushes after update.
pub async fn update_issue(
    project_path: &Path,
    issue_id: &str,
    options: UpdateIssueOptions,
) -> Result<UpdateIssueResult, IssueCrudError> {
    sync_before_read(project_path).await;

    let result = issue::update_issue(project_path, issue_id, options).await?;

    sync_after_write(
        project_path,
        &format!(
            "centy: Update issue #{} ({})",
            result.issue.metadata.display_number, issue_id
        ),
    )
    .await;

    Ok(result)
}

/// Delete an issue (sync-aware).
///
/// Commits and pushes after deletion.
pub async fn delete_issue(
    project_path: &Path,
    issue_id: &str,
) -> Result<DeleteIssueResult, IssueCrudError> {
    // Get display number before deletion for commit message
    let issue = issue::get_issue(project_path, issue_id).await.ok();
    let display_number = issue.map(|i| i.metadata.display_number);

    let result = issue::delete_issue(project_path, issue_id).await?;

    let msg = match display_number {
        Some(num) => format!("centy: Delete issue #{} ({})", num, issue_id),
        None => format!("centy: Delete issue {}", issue_id),
    };
    sync_after_write(project_path, &msg).await;

    Ok(result)
}

/// Soft-delete an issue (sync-aware).
///
/// Commits and pushes after soft-deletion.
pub async fn soft_delete_issue(
    project_path: &Path,
    issue_id: &str,
) -> Result<SoftDeleteIssueResult, IssueCrudError> {
    sync_before_read(project_path).await;

    let result = issue::soft_delete_issue(project_path, issue_id).await?;

    sync_after_write(
        project_path,
        &format!(
            "centy: Soft-delete issue #{} ({})",
            result.issue.metadata.display_number, issue_id
        ),
    )
    .await;

    Ok(result)
}

/// Restore a soft-deleted issue (sync-aware).
///
/// Commits and pushes after restoration.
pub async fn restore_issue(
    project_path: &Path,
    issue_id: &str,
) -> Result<RestoreIssueResult, IssueCrudError> {
    sync_before_read(project_path).await;

    let result = issue::restore_issue(project_path, issue_id).await?;

    sync_after_write(
        project_path,
        &format!(
            "centy: Restore issue #{} ({})",
            result.issue.metadata.display_number, issue_id
        ),
    )
    .await;

    Ok(result)
}

/// Move an issue to another project (sync-aware).
///
/// Commits and pushes to both source and target projects.
pub async fn move_issue(options: MoveIssueOptions) -> Result<MoveIssueResult, IssueCrudError> {
    // Pull both projects
    sync_before_read(&options.source_project_path).await;
    sync_before_read(&options.target_project_path).await;

    let result = issue::move_issue(options.clone()).await?;

    // Sync both projects
    sync_after_write(
        &options.source_project_path,
        &format!(
            "centy: Move issue #{} to another project ({})",
            result.old_display_number, result.issue.id
        ),
    )
    .await;

    sync_after_write(
        &options.target_project_path,
        &format!(
            "centy: Receive issue #{} from another project ({})",
            result.issue.metadata.display_number, result.issue.id
        ),
    )
    .await;

    Ok(result)
}

/// Duplicate an issue (sync-aware).
///
/// Commits and pushes after duplication.
pub async fn duplicate_issue(
    options: DuplicateIssueOptions,
) -> Result<DuplicateIssueResult, IssueCrudError> {
    // Pull both projects if different
    sync_before_read(&options.source_project_path).await;
    if options.source_project_path != options.target_project_path {
        sync_before_read(&options.target_project_path).await;
    }

    let result = issue::duplicate_issue(options.clone()).await?;

    sync_after_write(
        &options.target_project_path,
        &format!(
            "centy: Duplicate issue #{} as #{}",
            result.original_issue_id, result.issue.metadata.display_number
        ),
    )
    .await;

    Ok(result)
}

// =============================================================================
// Doc CRUD wrappers
// =============================================================================

/// Get a single doc by its slug (sync-aware).
///
/// Pulls latest changes before reading.
pub async fn get_doc(project_path: &Path, slug: &str) -> Result<Doc, DocError> {
    sync_before_read(project_path).await;
    docs::get_doc(project_path, slug).await
}

/// List all docs (sync-aware).
///
/// Pulls latest changes before reading.
pub async fn list_docs(project_path: &Path, include_deleted: bool) -> Result<Vec<Doc>, DocError> {
    sync_before_read(project_path).await;
    docs::list_docs(project_path, include_deleted).await
}

/// Create a new doc (sync-aware).
///
/// Commits and pushes after creation.
pub async fn create_doc(
    project_path: &Path,
    options: CreateDocOptions,
) -> Result<CreateDocResult, DocError> {
    sync_before_read(project_path).await;

    let result = docs::create_doc(project_path, options).await?;

    sync_after_write(
        project_path,
        &format!("centy: Create doc '{}'", result.slug),
    )
    .await;

    Ok(result)
}

/// Update an existing doc (sync-aware).
///
/// Commits and pushes after update.
pub async fn update_doc(
    project_path: &Path,
    slug: &str,
    options: UpdateDocOptions,
) -> Result<UpdateDocResult, DocError> {
    sync_before_read(project_path).await;

    let result = docs::update_doc(project_path, slug, options).await?;

    sync_after_write(project_path, &format!("centy: Update doc '{}'", slug)).await;

    Ok(result)
}

/// Delete a doc (sync-aware).
///
/// Commits and pushes after deletion.
pub async fn delete_doc(project_path: &Path, slug: &str) -> Result<DeleteDocResult, DocError> {
    let result = docs::delete_doc(project_path, slug).await?;

    sync_after_write(project_path, &format!("centy: Delete doc '{}'", slug)).await;

    Ok(result)
}

/// Soft-delete a doc (sync-aware).
///
/// Commits and pushes after soft-deletion.
pub async fn soft_delete_doc(
    project_path: &Path,
    slug: &str,
) -> Result<SoftDeleteDocResult, DocError> {
    sync_before_read(project_path).await;

    let result = docs::soft_delete_doc(project_path, slug).await?;

    sync_after_write(project_path, &format!("centy: Soft-delete doc '{}'", slug)).await;

    Ok(result)
}

/// Restore a soft-deleted doc (sync-aware).
///
/// Commits and pushes after restoration.
pub async fn restore_doc(project_path: &Path, slug: &str) -> Result<RestoreDocResult, DocError> {
    sync_before_read(project_path).await;

    let result = docs::restore_doc(project_path, slug).await?;

    sync_after_write(project_path, &format!("centy: Restore doc '{}'", slug)).await;

    Ok(result)
}

/// Move a doc to another project (sync-aware).
///
/// Commits and pushes to both source and target projects.
pub async fn move_doc(options: MoveDocOptions) -> Result<MoveDocResult, DocError> {
    sync_before_read(&options.source_project_path).await;
    sync_before_read(&options.target_project_path).await;

    let result = docs::move_doc(options.clone()).await?;

    sync_after_write(
        &options.source_project_path,
        &format!("centy: Move doc '{}' to another project", result.doc.slug),
    )
    .await;

    sync_after_write(
        &options.target_project_path,
        &format!(
            "centy: Receive doc '{}' from another project",
            result.doc.slug
        ),
    )
    .await;

    Ok(result)
}

/// Duplicate a doc (sync-aware).
///
/// Commits and pushes after duplication.
pub async fn duplicate_doc(options: DuplicateDocOptions) -> Result<DuplicateDocResult, DocError> {
    sync_before_read(&options.source_project_path).await;
    if options.source_project_path != options.target_project_path {
        sync_before_read(&options.target_project_path).await;
    }

    let result = docs::duplicate_doc(options.clone()).await?;

    sync_after_write(
        &options.target_project_path,
        &format!(
            "centy: Duplicate doc '{}' as '{}'",
            result.original_slug, result.doc.slug
        ),
    )
    .await;

    Ok(result)
}

// =============================================================================
// PR CRUD wrappers
// =============================================================================

/// Get a single PR by its ID (sync-aware).
///
/// Pulls latest changes before reading.
pub async fn get_pr(project_path: &Path, pr_id: &str) -> Result<PullRequest, PrCrudError> {
    sync_before_read(project_path).await;
    pr::get_pr(project_path, pr_id).await
}

/// Get a PR by its display number (sync-aware).
///
/// Pulls latest changes before reading.
pub async fn get_pr_by_display_number(
    project_path: &Path,
    display_number: u32,
) -> Result<PullRequest, PrCrudError> {
    sync_before_read(project_path).await;
    pr::get_pr_by_display_number(project_path, display_number).await
}

/// List all PRs (sync-aware).
///
/// Pulls latest changes before reading.
pub async fn list_prs(
    project_path: &Path,
    status_filter: Option<&str>,
    source_branch_filter: Option<&str>,
    target_branch_filter: Option<&str>,
    priority_filter: Option<u32>,
    include_deleted: bool,
) -> Result<Vec<PullRequest>, PrCrudError> {
    sync_before_read(project_path).await;
    pr::list_prs(
        project_path,
        status_filter,
        source_branch_filter,
        target_branch_filter,
        priority_filter,
        include_deleted,
    )
    .await
}

/// Create a new PR (sync-aware).
///
/// Commits and pushes after creation.
pub async fn create_pr(
    project_path: &Path,
    options: CreatePrOptions,
) -> Result<CreatePrResult, PrError> {
    sync_before_read(project_path).await;

    let result = pr::create_pr(project_path, options).await?;

    sync_after_write(
        project_path,
        &format!("centy: Create PR #{}", result.display_number),
    )
    .await;

    Ok(result)
}

/// Update an existing PR (sync-aware).
///
/// Commits and pushes after update.
pub async fn update_pr(
    project_path: &Path,
    pr_id: &str,
    options: UpdatePrOptions,
) -> Result<UpdatePrResult, PrCrudError> {
    sync_before_read(project_path).await;

    let result = pr::update_pr(project_path, pr_id, options).await?;

    sync_after_write(
        project_path,
        &format!(
            "centy: Update PR #{} ({})",
            result.pr.metadata.display_number, pr_id
        ),
    )
    .await;

    Ok(result)
}

/// Delete a PR (sync-aware).
///
/// Commits and pushes after deletion.
pub async fn delete_pr(project_path: &Path, pr_id: &str) -> Result<DeletePrResult, PrCrudError> {
    // Get display number before deletion for commit message
    let pr = pr::get_pr(project_path, pr_id).await.ok();
    let display_number = pr.map(|p| p.metadata.display_number);

    let result = pr::delete_pr(project_path, pr_id).await?;

    let msg = match display_number {
        Some(num) => format!("centy: Delete PR #{} ({})", num, pr_id),
        None => format!("centy: Delete PR {}", pr_id),
    };
    sync_after_write(project_path, &msg).await;

    Ok(result)
}

/// Soft-delete a PR (sync-aware).
///
/// Commits and pushes after soft-deletion.
pub async fn soft_delete_pr(
    project_path: &Path,
    pr_id: &str,
) -> Result<SoftDeletePrResult, PrCrudError> {
    sync_before_read(project_path).await;

    let result = pr::soft_delete_pr(project_path, pr_id).await?;

    sync_after_write(
        project_path,
        &format!(
            "centy: Soft-delete PR #{} ({})",
            result.pr.metadata.display_number, pr_id
        ),
    )
    .await;

    Ok(result)
}

/// Restore a soft-deleted PR (sync-aware).
///
/// Commits and pushes after restoration.
pub async fn restore_pr(project_path: &Path, pr_id: &str) -> Result<RestorePrResult, PrCrudError> {
    sync_before_read(project_path).await;

    let result = pr::restore_pr(project_path, pr_id).await?;

    sync_after_write(
        project_path,
        &format!(
            "centy: Restore PR #{} ({})",
            result.pr.metadata.display_number, pr_id
        ),
    )
    .await;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_sync_manager_non_git() {
        // Test that a non-git directory returns None
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = get_sync_manager(temp_dir.path()).await;
        assert!(manager.is_none());
    }
}
