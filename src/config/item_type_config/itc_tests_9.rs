use super::*;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_registry_build_with_valid_configs() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let issues_dir = centy_dir.join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_dir.join("config.yaml"), &yaml)
        .await
        .unwrap();

    let docs_dir = centy_dir.join("docs");
    fs::create_dir_all(&docs_dir).await.unwrap();
    let doc_config = default_doc_config();
    let yaml = serde_yaml::to_string(&doc_config).unwrap();
    fs::write(docs_dir.join("config.yaml"), &yaml)
        .await
        .unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

    assert_eq!(registry.len(), 2);
    assert!(!registry.is_empty());

    let issues = registry.get("issues").expect("Should have issues");
    assert_eq!(issues.name, "Issue");
    assert_eq!(issues.icon, Some("clipboard".to_string()));

    let docs = registry.get("docs").expect("Should have docs");
    assert_eq!(docs.name, "Doc");
}

#[tokio::test]
async fn test_registry_build_skips_dirs_without_config() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let issues_dir = centy_dir.join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_dir.join("config.yaml"), &yaml)
        .await
        .unwrap();

    fs::create_dir_all(centy_dir.join("assets")).await.unwrap();
    fs::create_dir_all(centy_dir.join("templates"))
        .await
        .unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
    assert_eq!(registry.len(), 1);
    assert!(registry.get("assets").is_none());
    assert!(registry.get("templates").is_none());
}
