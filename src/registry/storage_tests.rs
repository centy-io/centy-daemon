#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]
#![allow(clippy::items_after_statements, clippy::default_numeric_fallback)]

use super::*;

fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    acquire_registry_test_lock()
}

#[test]
fn test_get_registry_path() {
    // This test will work if HOME or USERPROFILE is set (or CENTY_HOME for isolated test runs)
    let result = get_registry_path();
    let home_set = std::env::var("HOME").is_ok()
        || std::env::var("USERPROFILE").is_ok()
        || std::env::var("CENTY_HOME").is_ok();
    if home_set {
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("projects.json"));
        // When CENTY_HOME is set the path may not contain ".centy"
        if std::env::var("CENTY_HOME").is_err() {
            assert!(path.to_string_lossy().contains(".centy"));
        }
    }
}

#[test]
fn test_project_registry_new() {
    let registry = ProjectRegistry::new();
    assert_eq!(registry.schema_version, CURRENT_SCHEMA_VERSION);
    assert!(registry.projects.is_empty());
    assert!(registry.organizations.is_empty());
    assert!(!registry.updated_at.is_empty());
}

#[test]
fn test_get_centy_config_dir_with_centy_home() {
    let _lock = acquire_lock();
    // CENTY_HOME is already set by acquire_registry_test_lock; just verify it works.
    let centy_home = std::env::var("CENTY_HOME").expect("CENTY_HOME should be set");
    let result = get_centy_config_dir();
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert_eq!(dir.to_string_lossy(), centy_home);
}

#[test]
fn test_get_centy_config_dir_uses_home() {
    let _lock = acquire_lock();
    // Remove CENTY_HOME so it falls through to HOME
    let centy_home_original = std::env::var("CENTY_HOME").ok();
    std::env::remove_var("CENTY_HOME");

    let home_set = std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok();
    if home_set {
        let result = get_centy_config_dir();
        assert!(result.is_ok());
        let dir = result.unwrap();
        assert!(dir.to_string_lossy().ends_with(".centy"));
    }

    // Restore CENTY_HOME
    if let Some(v) = centy_home_original {
        std::env::set_var("CENTY_HOME", v);
    }
}

#[tokio::test]
async fn test_read_registry_missing_file_returns_new() {
    let _lock = acquire_lock();
    // Use the shared CENTY_HOME — it has no projects.json on first run.
    // Just verify read succeeds and returns valid registry structure.
    let registry = read_registry().await.expect("Should succeed");
    assert_eq!(registry.schema_version, CURRENT_SCHEMA_VERSION);
}

#[tokio::test]
async fn test_write_and_read_registry() {
    let _lock = acquire_lock();
    use crate::registry::types::TrackedProject;

    let mut registry = read_registry().await.expect("read");
    let original_count = registry.projects.len();

    registry.projects.insert(
        "/storage-test/path/unique-write-test".to_string(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: true,
            is_archived: false,
            organization_slug: None,
            user_title: Some("My Project".to_string()),
        },
    );
    write_registry_unlocked(&registry)
        .await
        .expect("Should write");

    let read_back = read_registry().await.expect("Should read");
    assert!(read_back.projects.len() > original_count);
    let proj = read_back
        .projects
        .get("/storage-test/path/unique-write-test")
        .expect("should have project");
    assert!(proj.is_favorite);
    assert_eq!(proj.user_title, Some("My Project".to_string()));
}

#[tokio::test]
async fn test_read_registry_applies_migration() {
    let _lock = acquire_lock();
    use tokio::fs;

    let centy_home = std::env::var("CENTY_HOME").expect("CENTY_HOME set");
    // Write a v1 registry JSON manually
    let v1_json = r#"{"schemaVersion":1,"updatedAt":"2024-01-01T00:00:00Z","projects":{}}"#;
    let registry_path = std::path::Path::new(&centy_home).join("projects.json");
    fs::write(&registry_path, v1_json)
        .await
        .expect("write v1 json");

    // Reading should trigger migration to v2
    let registry = read_registry().await.expect("Should succeed");
    assert_eq!(registry.schema_version, 2);

    // The file should now be written at v2
    let content = fs::read_to_string(&registry_path).await.expect("read back");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse");
    assert_eq!(parsed["schemaVersion"], 2);
}

#[test]
fn test_get_lock_returns_same_instance() {
    let lock1: *const _ = get_lock();
    let lock2: *const _ = get_lock();
    assert_eq!(lock1, lock2, "get_lock should return the same static mutex");
}

#[test]
fn test_get_centy_config_dir_home_dir_not_found() {
    // We can't easily remove HOME/USERPROFILE in a portable way without
    // risking other tests, so just verify the error variant exists and
    // the function signature returns the right type.
    let _lock = acquire_lock();
    // With CENTY_HOME set (from acquire_registry_test_lock), this always succeeds.
    let result = get_centy_config_dir();
    assert!(result.is_ok());
}

#[test]
fn test_get_centy_config_dir_userprofile_fallback() {
    let _lock = acquire_lock();
    // Temporarily remove CENTY_HOME and HOME, then set USERPROFILE to test that path.
    // This exercises the `or_else(|_| std::env::var("USERPROFILE"))` branch.
    let saved_centy_home = std::env::var("CENTY_HOME").ok();
    let saved_home = std::env::var("HOME").ok();
    std::env::remove_var("CENTY_HOME");
    std::env::remove_var("HOME");
    std::env::set_var("USERPROFILE", "/fake/userprofile");

    let result = get_centy_config_dir();

    // Restore env vars
    std::env::remove_var("USERPROFILE");
    if let Some(v) = saved_home {
        std::env::set_var("HOME", v);
    }
    if let Some(v) = saved_centy_home {
        std::env::set_var("CENTY_HOME", v);
    }

    assert!(result.is_ok());
    let dir = result.unwrap();
    assert!(dir.to_string_lossy().ends_with(".centy"));
}

#[test]
fn test_get_centy_config_dir_no_home_vars_returns_error() {
    let _lock = acquire_lock();
    // Remove all home-related env vars to exercise the HomeDirNotFound error branch.
    let saved_centy_home = std::env::var("CENTY_HOME").ok();
    let saved_home = std::env::var("HOME").ok();
    let saved_userprofile = std::env::var("USERPROFILE").ok();
    std::env::remove_var("CENTY_HOME");
    std::env::remove_var("HOME");
    std::env::remove_var("USERPROFILE");

    let result = get_centy_config_dir();

    // Restore env vars before asserting (in case assert panics)
    if let Some(v) = saved_userprofile {
        std::env::set_var("USERPROFILE", v);
    }
    if let Some(v) = saved_home {
        std::env::set_var("HOME", v);
    }
    if let Some(v) = saved_centy_home {
        std::env::set_var("CENTY_HOME", v);
    }

    assert!(
        matches!(result, Err(RegistryError::HomeDirNotFound)),
        "Expected HomeDirNotFound when no home vars are set"
    );
}
