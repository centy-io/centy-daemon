use super::*;
use tempfile::TempDir;

#[test]
fn test_assert_initialized_fails_for_empty_dir() {
    let dir = TempDir::new().unwrap();
    let err = assert_initialized(dir.path()).unwrap_err();
    assert!(matches!(err, AssertError::NotInitialized));
}

#[test]
fn test_assert_initialized_passes_when_manifest_exists() {
    let dir = TempDir::new().unwrap();
    let centy_dir = dir.path().join(".centy");
    std::fs::create_dir_all(&centy_dir).unwrap();
    std::fs::write(centy_dir.join(".centy-manifest.json"), b"{}").unwrap();
    assert_initialized(dir.path()).unwrap();
}

#[test]
fn test_assert_initialized_fails_when_only_centy_dir_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(".centy")).unwrap();
    let err = assert_initialized(dir.path()).unwrap_err();
    assert!(matches!(err, AssertError::NotInitialized));
}

#[test]
fn test_assert_error_display() {
    let err = AssertError::NotInitialized;
    let msg = err.to_string();
    assert!(msg.contains("not initialized"));
    assert!(msg.contains(".centy-manifest.json"));
}

#[test]
fn test_assert_absolute_path_accepts_absolute() {
    assert_absolute_path("/Users/someone/project").unwrap();
}

#[test]
fn test_assert_absolute_path_rejects_relative() {
    let err = assert_absolute_path("relative/path").unwrap_err();
    assert!(matches!(err, AssertError::RelativePath(_)));
}

#[test]
fn test_assert_absolute_path_rejects_empty() {
    let err = assert_absolute_path("").unwrap_err();
    assert!(matches!(err, AssertError::RelativePath(_)));
}

#[test]
fn test_assert_absolute_path_rejects_dot() {
    let err = assert_absolute_path("./project").unwrap_err();
    assert!(matches!(err, AssertError::RelativePath(_)));
}

#[test]
fn test_relative_path_error_display() {
    let err = AssertError::RelativePath("my/project".to_string());
    let msg = err.to_string();
    assert!(msg.contains("absolute path"));
    assert!(msg.contains("my/project"));
}
