//! User management module.
//!
//! This module provides functionality for managing project users:
//! - Creating, reading, updating, and deleting users
//! - Syncing users from git history
//!
//! Users are stored in `.centy/users.json` in each project.

mod crud;
mod git;
mod storage;
mod sync;
mod types;

pub use crud::{
    create_user, delete_user, get_user, list_users, restore_user, soft_delete_user, update_user,
    CreateUserOptions, CreateUserResult, DeleteUserResult, RestoreUserResult, SoftDeleteUserResult,
    UpdateUserOptions, UpdateUserResult,
};
pub use git::{get_git_contributors, is_git_repository};
pub use storage::{find_user_by_email, find_user_by_id, read_users, write_users};
pub use sync::{sync_users, SyncUsersFullResult};
pub use types::{
    slugify, validate_user_id, GitContributor, SyncUsersResult, User, UserError, UsersFile,
};
