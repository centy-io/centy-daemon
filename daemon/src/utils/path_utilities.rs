use super::*;
use std::path::Path;

#[test]
fn test_get_centy_path() {
    let project_path = Path::new("/home/user/my-project");
    let centy_path = get_centy_path(project_path);

    assert_eq!(centy_path, Path::new("/home/user/my-project/.centy"));
}

#[test]
fn test_get_manifest_path() {
    let project_path = Path::new("/home/user/my-project");
    let manifest_path = get_manifest_path(project_path);

    assert_eq!(
        manifest_path,
        Path::new("/home/user/my-project/.centy/.centy-manifest.json")
    );
}

#[test]
fn test_centy_folder_constant() {
    assert_eq!(CENTY_FOLDER, ".centy");
}

#[test]
fn test_manifest_file_constant() {
    assert_eq!(MANIFEST_FILE, ".centy-manifest.json");
}

#[test]
fn test_centy_version_constant() {
    // Version should match Cargo.toml
    assert_eq!(CENTY_VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_now_iso_format() {
    let timestamp = now_iso();

    // Should be a valid RFC3339 timestamp
    assert!(timestamp.len() > 20, "Timestamp should be reasonably long");

    // Should contain date separators
    assert!(timestamp.contains('-'), "Should contain date separator");
    assert!(timestamp.contains(':'), "Should contain time separator");

    // Should be parseable
    let parsed = chrono::DateTime::parse_from_rfc3339(&timestamp);
    assert!(parsed.is_ok(), "Should be valid RFC3339 format");
}

#[test]
fn test_get_centy_path_relative() {
    let project_path = Path::new(".");
    let centy_path = get_centy_path(project_path);

    assert_eq!(centy_path, Path::new("./.centy"));
}

#[test]
fn test_paths_are_consistent() {
    let project_path = Path::new("/test");
    let centy_path = get_centy_path(project_path);
    let manifest_path = get_manifest_path(project_path);

    // Manifest path should be inside centy path
    assert!(manifest_path.starts_with(&centy_path));
}
