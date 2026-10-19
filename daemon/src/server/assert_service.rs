//! Assert service for enforcing preconditions before command execution.
//!
//! Provides composable assertion functions that handlers call to ensure required
//! invariants hold before performing their core logic. All assertions are fast
//! (single filesystem stat) and do not mutate state.
//!
//! # Design
//!
//! Each assertion function checks one specific precondition and returns
//! `Ok(())` on success or an [`AssertError`] on failure.  Handlers compose
//! assertions by calling whichever subset they require:
//!
//! ```ignore
//! assert_initialized(project_path)?;
//! ```
//!
//! `init` itself and daemon-level RPCs are exempt from these checks.

use std::fmt;
use std::path::Path;

use crate::utils::get_manifest_path;

/// Errors returned when an assertion precondition is not satisfied.
#[derive(Debug)]
pub enum AssertError {
    /// The project has not been initialized (`.centy-manifest.json` is absent).
    NotInitialized,
    /// The supplied `project_path` is not an absolute path.
    RelativePath(String),
}

impl fmt::Display for AssertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssertError::NotInitialized => write!(
                f,
                "project is not initialized: .centy-manifest.json not found"
            ),
            AssertError::RelativePath(p) => write!(
                f,
                "Invalid path: projectPath must be an absolute path, got: {p}"
            ),
        }
    }
}

/// Assert that `project_path` is absolute, so the daemon locates the project
/// on disk unambiguously instead of resolving against its own working directory.
///
/// # Errors
///
/// Returns [`AssertError::RelativePath`] when the path is not absolute.
pub fn assert_absolute_path(project_path: &Path) -> Result<(), AssertError> {
    if project_path.is_absolute() {
        Ok(())
    } else {
        Err(AssertError::RelativePath(
            project_path.to_string_lossy().into_owned(),
        ))
    }
}

/// Assert that the project at `project_path` has been initialized.
///
/// Also rejects relative paths (see [`assert_absolute_path`]); every handler
/// that gates on this function therefore refuses them.
///
/// Initialization is determined by the presence of `.centy/.centy-manifest.json`.
/// This is a fast check (single filesystem stat) and does not modify any files.
///
/// # Errors
///
/// Returns [`AssertError::NotInitialized`] if the manifest file does not exist.
pub fn assert_initialized(project_path: &Path) -> Result<(), AssertError> {
    assert_absolute_path(project_path)?;
    if get_manifest_path(project_path).exists() {
        Ok(())
    } else {
        Err(AssertError::NotInitialized)
    }
}

#[cfg(test)]
#[path = "assert_service_tests.rs"]
mod tests;
