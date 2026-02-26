//! Display number reconciliation for resolving conflicts.
//!
//! When multiple users create issues offline, they may assign the same display
//! number. This module detects and resolves such conflicts.
mod reconcile_fn;
mod scan;
mod types;
#[allow(unused_imports)]
pub use super::metadata::IssueMetadata;
pub use reconcile_fn::reconcile_display_numbers;
pub use scan::get_next_display_number;
#[allow(unused_imports)]
pub use std::path::Path;
#[allow(unused_imports)]
pub use tokio::fs;
pub use types::ReconcileError;
#[cfg(test)]
#[path = "../reconcile_tests_1.rs"]
mod reconcile_tests_1;
#[cfg(test)]
#[path = "../reconcile_tests_2.rs"]
mod reconcile_tests_2;
#[cfg(test)]
#[path = "../reconcile_tests_3.rs"]
mod reconcile_tests_3;
