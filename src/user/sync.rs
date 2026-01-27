//! Git user sync functionality.
//!
//! Syncs users from git history by extracting author names and emails
//! from git log and creating corresponding user entries.

use super::crud::{create_user, CreateUserOptions};
use super::git::{get_git_contributors, is_git_repository};
use super::storage::{find_user_by_email, read_users};
use super::types::{slugify, SyncUsersResult, UserError};
use crate::manifest::{read_manifest, update_manifest_timestamp, write_manifest, CentyManifest};
use std::path::Path;
use tracing::info;

/// Result of syncing users including the manifest
pub struct SyncUsersFullResult {
    pub result: SyncUsersResult,
    pub manifest: CentyManifest,
}

/// Sync users from git history.
///
/// Reads git log to find all contributors and creates user entries
/// for those who don't already exist (matched by email).
///
/// If `dry_run` is true, returns what would be created/skipped without
/// actually creating any users.
pub async fn sync_users(
    project_path: &Path,
    dry_run: bool,
) -> Result<SyncUsersFullResult, UserError> {
    // Read manifest to verify initialization
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;

    // Check if git repository
    if !is_git_repository(project_path) {
        return Err(UserError::NotGitRepository);
    }

    // Get contributors from git
    let contributors = get_git_contributors(project_path)?;

    // Read existing users
    let existing_users = read_users(project_path).await?;

    let mut result = SyncUsersResult::default();

    for contributor in contributors {
        // Check if user already exists by email
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
            // Generate slug from name
            let id = slugify(&contributor.name);

            // Create user
            match create_user(
                project_path,
                CreateUserOptions {
                    id: id.clone(),
                    name: contributor.name.clone(),
                    email: Some(contributor.email.clone()),
                    git_usernames: vec![contributor.name.clone()],
                },
            )
            .await
            {
                Ok(_) => {
                    result.created.push(id);
                }
                Err(e) => {
                    // If user already exists with this ID, try with email suffix
                    if matches!(e, UserError::UserAlreadyExists(_)) {
                        let email_slug =
                            slugify(contributor.email.split('@').next().unwrap_or("user"));
                        let fallback_id = format!("{id}-{email_slug}");

                        match create_user(
                            project_path,
                            CreateUserOptions {
                                id: fallback_id.clone(),
                                name: contributor.name.clone(),
                                email: Some(contributor.email.clone()),
                                git_usernames: vec![contributor.name.clone()],
                            },
                        )
                        .await
                        {
                            Ok(_) => {
                                result.created.push(fallback_id);
                            }
                            Err(e2) => {
                                result.errors.push(format!(
                                    "Failed to create user for {}: {}",
                                    contributor.email, e2
                                ));
                            }
                        }
                    } else {
                        result.errors.push(format!(
                            "Failed to create user for {}: {}",
                            contributor.email, e
                        ));
                    }
                }
            }
        }
    }

    // Update manifest timestamp if we made changes
    if !dry_run && !result.created.is_empty() {
        update_manifest_timestamp(&mut manifest);
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
