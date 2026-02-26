//! User read operations (create, get, list).
use super::types::{CreateUserOptions, CreateUserResult};
use super::super::storage::{find_user_by_id, read_users, write_users};
use super::super::types::{slugify, validate_user_id, User, UserError};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::now_iso;
use std::path::Path;
use tracing::info;
/// Create a new user
pub async fn create_user(project_path: &Path, options: CreateUserOptions) -> Result<CreateUserResult, UserError> {
    let id = if options.id.is_empty() { slugify(&options.name) } else { options.id };
    validate_user_id(&id)?;
    let mut manifest = read_manifest(project_path).await
        .map_err(|_| UserError::NotInitialized)?.ok_or(UserError::NotInitialized)?;
    let mut users = read_users(project_path).await?;
    if find_user_by_id(&users, &id).is_some() { return Err(UserError::UserAlreadyExists(id)); }
    let now = now_iso();
    let user = User {
        id: id.clone(), name: options.name, email: options.email,
        git_usernames: options.git_usernames,
        created_at: now.clone(), updated_at: now, deleted_at: None,
    };
    users.push(user.clone());
    users.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    write_users(project_path, &users).await?;
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    info!("Created user: {}", id);
    Ok(CreateUserResult { user, manifest })
}
/// Get a user by ID
pub async fn get_user(project_path: &Path, user_id: &str) -> Result<User, UserError> {
    let users = read_users(project_path).await?;
    users.into_iter().find(|u| u.id == user_id)
        .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))
}
/// List all users, optionally filtered by git username
pub async fn list_users(
    project_path: &Path, git_username_filter: Option<&str>, include_deleted: bool,
) -> Result<Vec<User>, UserError> {
    let users = read_users(project_path).await?;
    let users = if include_deleted { users } else {
        users.into_iter().filter(|u| u.deleted_at.is_none()).collect()
    };
    if let Some(filter) = git_username_filter {
        Ok(users.into_iter().filter(|u| u.git_usernames.iter().any(|g| g == filter)).collect())
    } else {
        Ok(users)
    }
}
