//! User management module.
//!
//! This module provides functionality for managing project users:
//! - Creating, reading, updating, and deleting users
//! - Syncing users from git history
//!
//! Users are stored in `.centy/users.json` in each project.

mod crud;
mod storage;
mod sync;
mod types;

#[allow(unused_imports)]
pub use crud::{
    create_user, delete_user, get_user, list_users, update_user,
    soft_delete_user, restore_user,
    CreateUserOptions, CreateUserResult, DeleteUserResult, UpdateUserOptions, UpdateUserResult,
    SoftDeleteUserResult, RestoreUserResult,
};
#[allow(unused_imports)]
pub use storage::{find_user_by_email, find_user_by_id, read_users, write_users};
#[allow(unused_imports)]
pub use sync::{sync_users, SyncUsersFullResult};
#[allow(unused_imports)]
pub use types::{slugify, validate_user_id, GitContributor, SyncUsersResult, User, UserError, UsersFile};
