//! Display number reconciliation for resolving conflicts.
//!
//! When multiple users create issues offline, they may assign the same display
//! number. This module detects and resolves such conflicts.
mod types;
mod scan;
mod reconcile_fn;
pub use types::ReconcileError;
pub use scan::get_next_display_number;
pub use reconcile_fn::reconcile_display_numbers;
#[allow(unused_imports)]
pub use super::metadata::IssueMetadata;
#[allow(unused_imports)]
pub use std::path::Path;
#[allow(unused_imports)]
pub use tokio::fs;
#[cfg(test)]
#[path = "../reconcile_tests_1.rs"]
mod reconcile_tests_1;
#[cfg(test)]
#[path = "../reconcile_tests_2.rs"]
mod reconcile_tests_2;
#[cfg(test)]
#[path = "../reconcile_tests_3.rs"]
mod reconcile_tests_3;
