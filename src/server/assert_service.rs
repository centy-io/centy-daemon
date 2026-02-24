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
//! assert_initialized(project_path).await?;
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
}

impl fmt::Display for AssertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssertError::NotInitialized => write!(
                f,
                "project is not initialized: .centy-manifest.json not found"
            ),
        }
    }
}

/// Assert that the project at `project_path` has been initialized.
///
/// Initialization is determined by the presence of `.centy/.centy-manifest.json`.
/// This is a fast check (single filesystem stat) and does not modify any files.
///
/// # Errors
///
/// Returns [`AssertError::NotInitialized`] if the manifest file does not exist.
pub async fn assert_initialized(project_path: &Path) -> Result<(), AssertError> {
    if get_manifest_path(project_path).exists() {
        Ok(())
    } else {
        Err(AssertError::NotInitialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_assert_initialized_fails_for_empty_dir() {
        let dir = TempDir::new().unwrap();
        let err = assert_initialized(dir.path()).await.unwrap_err();
        assert!(matches!(err, AssertError::NotInitialized));
    }

    #[tokio::test]
    async fn test_assert_initialized_passes_when_manifest_exists() {
        let dir = TempDir::new().unwrap();
        let centy_dir = dir.path().join(".centy");
        std::fs::create_dir_all(&centy_dir).unwrap();
        std::fs::write(centy_dir.join(".centy-manifest.json"), b"{}").unwrap();
        assert_initialized(dir.path()).await.unwrap();
    }

    #[tokio::test]
    async fn test_assert_initialized_fails_when_only_centy_dir_exists() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".centy")).unwrap();
        let err = assert_initialized(dir.path()).await.unwrap_err();
        assert!(matches!(err, AssertError::NotInitialized));
    }

    #[test]
    fn test_assert_error_display() {
        let err = AssertError::NotInitialized;
        let msg = err.to_string();
        assert!(msg.contains("not initialized"));
        assert!(msg.contains(".centy-manifest.json"));
    }
}
