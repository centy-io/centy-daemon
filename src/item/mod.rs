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
pub mod validation;

// Re-export core types
pub use core::error::ItemError;
pub use core::id::{Identifiable, ItemId};
pub use core::metadata::ItemMetadata;

// Re-export organization types
pub use organization::org_sync::{
    sync_to_org_projects, sync_update_to_org_projects, OrgSyncError, OrgSyncResult, OrgSyncable,
};

/// Item type discriminator for API/serialization purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Issue,
    Doc,
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Issue => write!(f, "issue"),
            ItemType::Doc => write!(f, "doc"),
        }
    }
}

impl std::str::FromStr for ItemType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "issue" => Ok(ItemType::Issue),
            "doc" | "docs" => Ok(ItemType::Doc),
            _ => Err(format!("Unknown item type: {s}")),
        }
    }
}
