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
