use super::super::types::User;
use crate::manifest::CentyManifest;
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
/// Result of deleting a user
pub struct DeleteUserResult {
    pub manifest: CentyManifest,
}
/// Result of soft-deleting a user
pub struct SoftDeleteUserResult {
    pub user: User,
    pub manifest: CentyManifest,
}
/// Result of restoring a soft-deleted user
pub struct RestoreUserResult {
    pub user: User,
    pub manifest: CentyManifest,
}
