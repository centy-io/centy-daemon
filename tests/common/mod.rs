#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Common test utilities

use std::path::Path;
use tempfile::TempDir;

/// Create a temporary directory for testing
pub fn create_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Initialize a centy project in the given directory by calling `execute_reconciliation`
#[allow(dead_code)] // Used across integration test files
pub async fn init_centy_project(project_path: &Path) {
    use centy_daemon::reconciliation::{execute_reconciliation, ReconciliationDecisions};

    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(project_path, decisions, true)
        .await
        .expect("Failed to initialize centy project");
}

/// Verify that the .centy folder exists with expected structure
#[allow(dead_code)] // Test utility for integration tests
pub fn verify_centy_structure(project_path: &Path) {
    let centy_path = project_path.join(".centy");
    assert!(centy_path.exists(), ".centy folder should exist");
    assert!(
        centy_path.join(".centy-manifest.json").exists(),
        "Manifest should exist"
    );
    assert!(centy_path.join("issues").exists(), "issues/ should exist");
    assert!(centy_path.join("docs").exists(), "docs/ should exist");
    assert!(
        centy_path.join("archived").exists(),
        "archived/ should exist"
    );
    assert!(centy_path.join("assets").exists(), "assets/ should exist");
    assert!(
        centy_path.join("README.md").exists(),
        "README.md should exist"
    );
    assert!(
        centy_path.join("config.json").exists(),
        "config.json should exist"
    );
    assert!(
        centy_path.join("issues").join("config.yaml").exists(),
        "issues/config.yaml should exist"
    );
    assert!(
        centy_path.join("docs").join("config.yaml").exists(),
        "docs/config.yaml should exist"
    );
    assert!(
        centy_path.join("archived").join("config.yaml").exists(),
        "archived/config.yaml should exist"
    );
}
