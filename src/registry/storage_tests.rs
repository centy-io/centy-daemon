use super::*;

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
