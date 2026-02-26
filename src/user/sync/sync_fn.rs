use super::super::crud::{create_user, CreateUserOptions};
use super::super::git::{get_git_contributors, is_git_repository};
use super::super::storage::{find_user_by_email, read_users};
use super::super::types::{slugify, SyncUsersResult, UserError};
use crate::manifest::{read_manifest, update_manifest, write_manifest, CentyManifest};
use std::path::Path;
use tracing::info;
/// Result of syncing users including the manifest
pub struct SyncUsersFullResult { pub result: SyncUsersResult, pub manifest: CentyManifest }
/// Sync users from git history.
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines, max_nesting_depth)]
pub async fn sync_users(project_path: &Path, dry_run: bool) -> Result<SyncUsersFullResult, UserError> {
    let mut manifest = read_manifest(project_path).await
        .map_err(|_| UserError::NotInitialized)?.ok_or(UserError::NotInitialized)?;
    if !is_git_repository(project_path) { return Err(UserError::NotGitRepository); }
    let contributors = get_git_contributors(project_path)?;
    let existing_users = read_users(project_path).await?;
    let mut result = SyncUsersResult::default();
    for contributor in contributors {
        if find_user_by_email(&existing_users, &contributor.email).is_some() {
            if dry_run { result.would_skip.push(contributor); } else { result.skipped.push(contributor.email); }
            continue;
        }
        if dry_run {
            result.would_create.push(contributor);
        } else {
            let id = slugify(&contributor.name);
            match create_user(project_path, CreateUserOptions {
                id: id.clone(), name: contributor.name.clone(),
                email: Some(contributor.email.clone()),
                git_usernames: vec![contributor.name.clone()],
            }).await {
                Ok(_) => { result.created.push(id); }
                Err(e) => {
                    if matches!(e, UserError::UserAlreadyExists(_)) {
                        let email_slug = slugify(contributor.email.split('@').next().unwrap_or("user"));
                        let fallback_id = format!("{id}-{email_slug}");
                        match create_user(project_path, CreateUserOptions {
                            id: fallback_id.clone(), name: contributor.name.clone(),
                            email: Some(contributor.email.clone()),
                            git_usernames: vec![contributor.name.clone()],
                        }).await {
                            Ok(_) => { result.created.push(fallback_id); }
                            Err(e2) => { result.errors.push(format!("Failed to create user for {}: {}", contributor.email, e2)); }
                        }
                    } else {
                        result.errors.push(format!("Failed to create user for {}: {}", contributor.email, e));
                    }
                }
            }
        }
    }
    if !dry_run && !result.created.is_empty() {
        update_manifest(&mut manifest);
        write_manifest(project_path, &manifest).await?;
    }
    info!("Synced users from git: {} created, {} skipped, {} errors",
        result.created.len(), result.skipped.len(), result.errors.len());
    Ok(SyncUsersFullResult { result, manifest })
}
