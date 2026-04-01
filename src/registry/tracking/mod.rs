mod counts;
mod enrich;
mod enrich_fn;
mod enrich_lookups;
mod ops;
mod set_ops;
#[cfg(test)]
pub use counts::{count_issues, count_md_files};
pub use enrich::{get_org_projects, list_projects};
#[cfg(test)]
pub use enrich_fn::is_version_behind;
pub use enrich_lookups::get_project_info;
pub use ops::{enrich_project, track_project, track_project_async, untrack_project};
pub use set_ops::{set_project_archived, set_project_favorite, set_project_user_title};
/// Acquire the shared registry test lock (delegates to `storage::acquire_registry_test_lock`).
/// Ensures all tracking tests share the same `CENTY_HOME` as other registry unit tests.
#[cfg(test)]
pub fn acquire_tracking_test_lock() -> std::sync::MutexGuard<'static, ()> {
    super::storage::acquire_registry_test_lock()
}
#[cfg(test)]
#[path = "../tracking_tests.rs"]
mod tracking_tests;
