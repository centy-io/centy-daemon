use super::super::git::{get_git_contributors, is_git_repository};
use super::super::storage::{find_user_by_email, read_users};
use super::super::types::{SyncUsersResult, UserError};
use super::helpers::create_user_from_contributor;
use crate::manifest::{read_manifest, update_manifest, write_manifest, CentyManifest};
use std::path::Path;
use tracing::info;
/// Result of syncing users including the manifest
pub struct SyncUsersFullResult {
    pub result: SyncUsersResult,
    pub manifest: CentyManifest,
}
/// Sync users from git history.
pub async fn sync_users(
    project_path: &Path,
    dry_run: bool,
) -> Result<SyncUsersFullResult, UserError> {
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_e| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;
    if !is_git_repository(project_path) {
        return Err(UserError::NotGitRepository);
    }
    let contributors = get_git_contributors(project_path)?;
    let existing_users = read_users(project_path).await?;
    let mut result = SyncUsersResult::default();
    for contributor in contributors {
        if find_user_by_email(&existing_users, &contributor.email).is_some() {
            if dry_run {
                result.would_skip.push(contributor);
            } else {
                result.skipped.push(contributor.email);
            }
            continue;
        }
        if dry_run {
            result.would_create.push(contributor);
        } else {
            create_user_from_contributor(project_path, &contributor, &mut result).await;
        }
    }
    if !dry_run && !result.created.is_empty() {
        update_manifest(&mut manifest);
        write_manifest(project_path, &manifest).await?;
    }
    info!(
        "Synced users from git: {} created, {} skipped, {} errors",
        result.created.len(),
        result.skipped.len(),
        result.errors.len()
    );
    Ok(SyncUsersFullResult { result, manifest })
}
