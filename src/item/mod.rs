//! Unified Item domain following Domain-Driven Design principles.
//!
//! This module consolidates Issues and Docs into a unified "item" concept
//! with shared traits for common operations while preserving entity-specific behavior.

// Allow unused code in this module - these are infrastructure traits/types
// that are prepared for future use but not yet fully integrated
#![allow(dead_code, unused_imports)]

pub mod core;
pub mod entities;
pub mod generic;
pub mod lifecycle;
pub mod operations;
pub mod organization;

// Re-export core types
pub use core::error::ItemError;
pub use core::metadata::ItemMetadata;

// Re-export organization types
pub use organization::org_sync::{
    sync_to_org_projects, sync_update_to_org_projects, OrgSyncError, OrgSyncResult, OrgSyncable,
};
