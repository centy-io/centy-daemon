//! Display number reconciliation for resolving conflicts.
//!
//! When multiple users create issues offline, they may assign the same display
//! number. This module detects and resolves such conflicts.
mod reconcile_fn;
mod scan;
mod types;
pub use super::metadata::IssueMetadata;
pub use reconcile_fn::reconcile_display_numbers;
pub use scan::get_next_display_number;
pub use std::path::Path;
pub use tokio::fs;
pub use types::ReconcileError;
#[cfg(test)]
#[path = "../reconcile_basic.rs"]
mod reconcile_basic;
#[cfg(test)]
#[path = "../reconcile_helpers.rs"]
mod reconcile_helpers;
#[cfg(test)]
#[path = "../reconcile_variants.rs"]
mod reconcile_variants;
