use super::super::crud::{create_user, update_user, CreateUserOptions, UpdateUserOptions};
use super::super::types::{slugify, GitContributor, SyncUsersResult, UserError};
use std::path::Path;

pub async fn update_user_git_usernames(
    project_path: &Path,
    user_id: &str,
    contributor: &GitContributor,
    result: &mut SyncUsersResult,
) {
    let options = UpdateUserOptions {
        name: None,
        email: None,
        git_usernames: Some(vec![contributor.name.clone()]),
    };
    match update_user(project_path, user_id, options).await {
        Ok(_) => result.updated.push(user_id.to_string()),
        Err(e) => result
            .errors
            .push(format!("Failed to update user {user_id}: {e}")),
    }
}

pub async fn create_user_from_contributor(
    project_path: &Path,
    contributor: &GitContributor,
    result: &mut SyncUsersResult,
) {
    let id = slugify(&contributor.name);
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
        Ok(_) => result.created.push(id),
        Err(e) => {
            if matches!(e, UserError::UserAlreadyExists(_)) {
                let email_slug = slugify(contributor.email.split('@').next().unwrap_or("user"));
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
                    Ok(_) => result.created.push(fallback_id),
                    Err(e2) => result.errors.push(format!(
                        "Failed to create user for {}: {}",
                        contributor.email, e2
                    )),
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
