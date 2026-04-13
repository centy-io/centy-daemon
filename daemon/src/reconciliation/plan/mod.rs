mod builder;
mod hashing;
pub mod managed_files;
mod types;
pub mod user_files;

pub use builder::build_reconciliation_plan;
pub use types::{FileInfo, PlanError, ReconciliationPlan};

#[cfg(test)]
use crate::manifest::ManagedFileType;
#[cfg(test)]
use crate::reconciliation::managed_files::get_managed_files;

#[cfg(test)]
#[path = "../plan_building.rs"]
mod plan_building;
#[cfg(test)]
#[path = "../plan_tests.rs"]
mod tests;
