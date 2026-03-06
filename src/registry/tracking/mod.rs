mod counts;
mod enrich;
mod enrich_fn;
mod enrich_lookups;
mod ops;
mod set_ops;
mod slug_check;
pub use enrich::{get_org_projects, list_projects};
#[cfg(test)]
pub use enrich_fn::is_version_behind;
pub use enrich_lookups::get_project_info;
pub use ops::{enrich_project, track_project, track_project_async, untrack_project};
pub use set_ops::{set_project_archived, set_project_favorite, set_project_user_title};
pub use slug_check::{find_duplicate_slugs, DuplicateSlugGroup};
#[cfg(test)]
#[path = "../slug_check_tests.rs"]
mod slug_check_tests;
#[cfg(test)]
#[path = "../tracking_tests.rs"]
mod tracking_tests;
