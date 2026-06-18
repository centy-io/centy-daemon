mod assignment;
mod create;
mod delete;
mod errors;
mod org_file;
mod query;
mod slug;
mod sync;
mod update;

pub use assignment::set_project_organization;
pub use create::create_organization;
pub use delete::delete_organization;
pub use errors::OrganizationError;
pub use query::{get_organization, list_organizations};
pub use slug::slugify;
pub use sync::sync_org_from_project;
pub use update::update_organization;

/// Acquire the shared registry test lock (delegates to `storage::acquire_registry_test_lock`).
/// Ensures all org tests share the same `CENTY_HOME` as other registry unit tests.
#[cfg(test)]
pub fn acquire_org_test_lock() -> std::sync::MutexGuard<'static, ()> {
    super::storage::acquire_registry_test_lock()
}
