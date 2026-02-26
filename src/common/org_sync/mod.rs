//! Cross-organization item synchronization traits and utilities.
mod types;
mod trait_def;
mod sync_fns;
pub use types::{OrgSyncError, OrgSyncResult};
pub use trait_def::OrgSyncable;
pub use sync_fns::{sync_to_org_projects, sync_update_to_org_projects};
#[cfg(test)]
#[path = "../org_sync_tests.rs"]
mod org_sync_tests;
