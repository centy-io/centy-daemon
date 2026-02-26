use super::*;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_registry_get_by_name() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let issues_dir = centy_dir.join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_dir.join("config.yaml"), &yaml).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

    let (folder, config) = registry.get_by_name("Issue").expect("Should find Issue");
    assert_eq!(folder, "issues");
    assert_eq!(config.name, "Issue");

    assert!(registry.get_by_name("NonExistent").is_none());
}

#[tokio::test]
async fn test_registry_folders() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let issues_dir = centy_dir.join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_dir.join("config.yaml"), &yaml).await.unwrap();

    let docs_dir = centy_dir.join("docs");
    fs::create_dir_all(&docs_dir).await.unwrap();
    let doc_config = default_doc_config();
    let yaml = serde_yaml::to_string(&doc_config).unwrap();
    fs::write(docs_dir.join("config.yaml"), &yaml).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
    let mut folders: Vec<&String> = registry.folders();
    folders.sort();

    assert_eq!(folders.len(), 2);
    assert_eq!(folders[0], "docs");
    assert_eq!(folders[1], "issues");
}

#[tokio::test]
async fn test_registry_all() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let issues_dir = centy_dir.join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_dir.join("config.yaml"), &yaml).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
    let all = registry.all();
    assert_eq!(all.len(), 1);
    assert!(all.contains_key("issues"));
}
