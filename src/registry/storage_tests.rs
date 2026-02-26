use super::*;

#[test]
fn test_get_registry_path() {
    // This test will work if HOME or USERPROFILE is set
    let result = get_registry_path();
    if std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok() {
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("projects.json"));
        assert!(path.to_string_lossy().contains(".centy"));
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
