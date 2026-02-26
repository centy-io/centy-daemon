//! Cross-organization item synchronization traits and utilities.
mod sync_fns;
mod trait_def;
mod types;
pub use sync_fns::{sync_to_org_projects, sync_update_to_org_projects};
pub use trait_def::OrgSyncable;
pub use types::{OrgSyncError, OrgSyncResult};
#[cfg(test)]
#[path = "../org_sync_tests.rs"]
mod org_sync_tests;
