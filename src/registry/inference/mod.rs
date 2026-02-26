//! Organization inference from git remote URLs.
mod auto_assign;
mod infer;
pub use auto_assign::try_auto_assign_organization;
pub use infer::{infer_organization_from_remote, OrgInferenceResult};
#[cfg(test)]
#[path = "../inference_tests.rs"]
mod inference_tests;
