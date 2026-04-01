use super::*;
use crate::config::CentyConfig;
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
    fs::write(issues_dir.join("config.yaml"), &yaml)
        .await
        .unwrap();

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
    fs::write(issues_dir.join("config.yaml"), &yaml)
        .await
        .unwrap();

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
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let yaml = serde_yaml::to_string(&epic_config).unwrap();
    fs::write(epics_dir.join("config.yaml"), &yaml)
        .await
        .unwrap();

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

#[tokio::test]
async fn test_registry_resolve_by_folder_case_insensitive_fallback() {
    // This tests the case-insensitive *folder* match path (the final fallback),
    // which triggers when exact folder and case-insensitive name both fail to match.
    // Folder = "my-bugs", Name = "BugReport" → resolving "my-bugs" (exact hits first),
    // but "MY-BUGS" won't match name "BugReport" and will fall through to folder compare.
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let bugs_dir = centy_dir.join("my-bugs");
    fs::create_dir_all(&bugs_dir).await.unwrap();
    let bug_config = ItemTypeConfig {
        name: "BugReport".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let yaml = serde_yaml::to_string(&bug_config).unwrap();
    fs::write(bugs_dir.join("config.yaml"), &yaml)
        .await
        .unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

    // "MY-BUGS" won't match exact folder "my-bugs", won't match name "BugReport",
    // but WILL match case-insensitive folder "my-bugs"
    let (folder, config) = registry
        .resolve("MY-BUGS")
        .expect("Should resolve via folder case-insensitive fallback");
    assert_eq!(folder, "my-bugs");
    assert_eq!(config.name, "BugReport");
}
