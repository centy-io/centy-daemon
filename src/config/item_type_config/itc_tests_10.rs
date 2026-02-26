use super::*;
use mdstore::IdStrategy;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_registry_resolve_by_folder() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let issues_dir = centy_dir.join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_dir.join("config.yaml"), &yaml).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

    let (folder, config) = registry.resolve("issues").expect("Should resolve 'issues'");
    assert_eq!(folder, "issues");
    assert_eq!(config.name, "Issue");
}

#[tokio::test]
async fn test_registry_resolve_by_name() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let issues_dir = centy_dir.join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_dir.join("config.yaml"), &yaml).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

    let (folder, _) = registry.resolve("issue").expect("Should resolve 'issue'");
    assert_eq!(folder, "issues");

    let (folder, _) = registry.resolve("Issue").expect("Should resolve 'Issue'");
    assert_eq!(folder, "issues");

    let (folder, _) = registry.resolve("ISSUE").expect("Should resolve 'ISSUE'");
    assert_eq!(folder, "issues");
}

#[tokio::test]
async fn test_registry_resolve_by_folder_case_insensitive() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let epics_dir = centy_dir.join("my-epics");
    fs::create_dir_all(&epics_dir).await.unwrap();
    let epic_config = ItemTypeConfig {
        name: "Epic".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let yaml = serde_yaml::to_string(&epic_config).unwrap();
    fs::write(epics_dir.join("config.yaml"), &yaml).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

    let (folder, _) = registry.resolve("epic").expect("Should resolve 'epic'");
    assert_eq!(folder, "my-epics");
}

#[tokio::test]
async fn test_registry_resolve_not_found() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");
    fs::create_dir_all(&centy_dir).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
    assert!(registry.resolve("nonexistent").is_none());
}
