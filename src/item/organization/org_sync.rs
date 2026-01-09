//! Cross-organization item synchronization traits and utilities.
//!
//! This module re-exports the org sync functionality from the common module
//! and provides additional abstractions for the item domain.

// Re-export from common module during transition
pub use crate::common::org_sync::{
    sync_to_org_projects, sync_update_to_org_projects, OrgSyncError, OrgSyncResult, OrgSyncable,
};
