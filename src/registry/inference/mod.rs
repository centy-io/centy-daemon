//! Organization inference from git remote URLs.
mod infer;
mod auto_assign;
pub use infer::{infer_organization_from_remote, OrgInferenceResult};
pub use auto_assign::try_auto_assign_organization;
#[cfg(test)]
#[path = "../inference_tests.rs"]
mod inference_tests;
