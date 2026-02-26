//! User write operations (update, delete, soft_delete, restore).
use super::super::storage::{read_users, write_users};
use super::super::types::UserError;
use super::types::{
    DeleteUserResult, RestoreUserResult, SoftDeleteUserResult, UpdateUserOptions, UpdateUserResult,
};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::now_iso;
use std::path::Path;
use tracing::info;
/// Update an existing user
pub async fn update_user(
    project_path: &Path,
    user_id: &str,
    options: UpdateUserOptions,
) -> Result<UpdateUserResult, UserError> {
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;
    let mut users = read_users(project_path).await?;
    let user = users
        .iter_mut()
        .find(|u| u.id == user_id)
        .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;
    if let Some(name) = options.name {
        if !name.is_empty() {
            user.name = name;
        }
    }
    if let Some(email) = options.email {
        user.email = if email.is_empty() { None } else { Some(email) };
    }
    if let Some(git_usernames) = options.git_usernames {
        if !git_usernames.is_empty() {
            user.git_usernames = git_usernames;
        }
    }
    user.updated_at = now_iso();
    let updated_user = user.clone();
    users.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    write_users(project_path, &users).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    info!("Updated user: {}", user_id);
    Ok(UpdateUserResult {
        user: updated_user,
        manifest,
    })
}
/// Delete a user
pub async fn delete_user(
    project_path: &Path,
    user_id: &str,
) -> Result<DeleteUserResult, UserError> {
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;
    let mut users = read_users(project_path).await?;
    let user_idx = users
        .iter()
        .position(|u| u.id == user_id)
        .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;
    users.remove(user_idx);
    write_users(project_path, &users).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    info!("Deleted user: {}", user_id);
    Ok(DeleteUserResult { manifest })
}
/// Soft-delete a user (set deleted_at timestamp)
pub async fn soft_delete_user(
    project_path: &Path,
    user_id: &str,
) -> Result<SoftDeleteUserResult, UserError> {
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;
    let mut users = read_users(project_path).await?;
    let user = users
        .iter_mut()
        .find(|u| u.id == user_id)
        .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;
    if user.deleted_at.is_some() {
        return Err(UserError::UserAlreadyDeleted(user_id.to_string()));
    }
    let now = now_iso();
    user.deleted_at = Some(now.clone());
    user.updated_at = now;
    let updated_user = user.clone();
    write_users(project_path, &users).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    info!("Soft-deleted user: {}", user_id);
    Ok(SoftDeleteUserResult {
        user: updated_user,
        manifest,
    })
}
/// Restore a soft-deleted user (clear deleted_at timestamp)
pub async fn restore_user(
    project_path: &Path,
    user_id: &str,
) -> Result<RestoreUserResult, UserError> {
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;
    let mut users = read_users(project_path).await?;
    let user = users
        .iter_mut()
        .find(|u| u.id == user_id)
        .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;
    if user.deleted_at.is_none() {
        return Err(UserError::UserNotDeleted(user_id.to_string()));
    }
    user.deleted_at = None;
    user.updated_at = now_iso();
    let restored_user = user.clone();
    write_users(project_path, &users).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    info!("Restored user: {}", user_id);
    Ok(RestoreUserResult {
        user: restored_user,
        manifest,
    })
}
