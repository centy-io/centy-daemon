//! User CRUD operations.

use super::storage::{find_user_by_id, read_users, write_users};
use super::types::{slugify, validate_user_id, User, UserError};
use crate::manifest::{read_manifest, update_manifest_timestamp, write_manifest, CentyManifest};
use crate::utils::now_iso;
use std::path::Path;
use tracing::info;

/// Options for creating a user
pub struct CreateUserOptions {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub git_usernames: Vec<String>,
}

/// Result of creating a user
pub struct CreateUserResult {
    pub user: User,
    pub manifest: CentyManifest,
}

/// Create a new user
pub async fn create_user(
    project_path: &Path,
    options: CreateUserOptions,
) -> Result<CreateUserResult, UserError> {
    // Validate ID
    let id = if options.id.is_empty() {
        slugify(&options.name)
    } else {
        options.id
    };
    validate_user_id(&id)?;

    // Read manifest
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;

    // Read existing users
    let mut users = read_users(project_path).await?;

    // Check if user already exists
    if find_user_by_id(&users, &id).is_some() {
        return Err(UserError::UserAlreadyExists(id));
    }

    // Create new user
    let now = now_iso();
    let user = User {
        id: id.clone(),
        name: options.name,
        email: options.email,
        git_usernames: options.git_usernames,
        created_at: now.clone(),
        updated_at: now,
    };

    users.push(user.clone());

    // Sort users by name
    users.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Write users
    write_users(project_path, &users).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    info!("Created user: {}", id);

    Ok(CreateUserResult { user, manifest })
}

/// Get a user by ID
pub async fn get_user(project_path: &Path, user_id: &str) -> Result<User, UserError> {
    let users = read_users(project_path).await?;

    users
        .into_iter()
        .find(|u| u.id == user_id)
        .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))
}

/// List all users, optionally filtered by git username
pub async fn list_users(
    project_path: &Path,
    git_username_filter: Option<&str>,
) -> Result<Vec<User>, UserError> {
    let users = read_users(project_path).await?;

    if let Some(filter) = git_username_filter {
        Ok(users
            .into_iter()
            .filter(|u| u.git_usernames.iter().any(|g| g == filter))
            .collect())
    } else {
        Ok(users)
    }
}

/// Options for updating a user
pub struct UpdateUserOptions {
    pub name: Option<String>,
    pub email: Option<String>,
    pub git_usernames: Option<Vec<String>>,
}

/// Result of updating a user
pub struct UpdateUserResult {
    pub user: User,
    pub manifest: CentyManifest,
}

/// Update an existing user
pub async fn update_user(
    project_path: &Path,
    user_id: &str,
    options: UpdateUserOptions,
) -> Result<UpdateUserResult, UserError> {
    // Read manifest
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;

    // Read existing users
    let mut users = read_users(project_path).await?;

    // Find and update user
    let user_idx = users
        .iter()
        .position(|u| u.id == user_id)
        .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;

    let user = &mut users[user_idx];

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

    // Sort users by name
    users.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Write users
    write_users(project_path, &users).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    info!("Updated user: {}", user_id);

    Ok(UpdateUserResult {
        user: updated_user,
        manifest,
    })
}

/// Result of deleting a user
pub struct DeleteUserResult {
    pub manifest: CentyManifest,
}

/// Delete a user
pub async fn delete_user(
    project_path: &Path,
    user_id: &str,
) -> Result<DeleteUserResult, UserError> {
    // Read manifest
    let mut manifest = read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;

    // Read existing users
    let mut users = read_users(project_path).await?;

    // Find user index
    let user_idx = users
        .iter()
        .position(|u| u.id == user_id)
        .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;

    // Remove user
    users.remove(user_idx);

    // Write users
    write_users(project_path, &users).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    info!("Deleted user: {}", user_id);

    Ok(DeleteUserResult { manifest })
}
