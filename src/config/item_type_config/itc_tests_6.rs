use super::*;
use mdstore::IdStrategy;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_registry_build_skips_malformed_yaml() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let issues_dir = centy_dir.join("issues");
    fs::create_dir_all(&issues_dir).await.unwrap();
    let issue_config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&issue_config).unwrap();
    fs::write(issues_dir.join("config.yaml"), &yaml)
        .await
        .unwrap();

    let bad_dir = centy_dir.join("broken");
    fs::create_dir_all(&bad_dir).await.unwrap();
    fs::write(
        bad_dir.join("config.yaml"),
        "this is: [not: valid: yaml: {{",
    )
    .await
    .unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

    assert_eq!(registry.len(), 1);
    assert!(registry.get("issues").is_some());
    assert!(registry.get("broken").is_none());
}

#[tokio::test]
async fn test_registry_build_detects_duplicate_type_names() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");

    let dir_a = centy_dir.join("aaa-issues");
    fs::create_dir_all(&dir_a).await.unwrap();
    let config_a = ItemTypeConfig {
        name: "Issue".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let yaml = serde_yaml::to_string(&config_a).unwrap();
    fs::write(dir_a.join("config.yaml"), &yaml).await.unwrap();

    let dir_b = centy_dir.join("zzz-issues");
    fs::create_dir_all(&dir_b).await.unwrap();
    let config_b = ItemTypeConfig {
        name: "Issue".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    };
    let yaml = serde_yaml::to_string(&config_b).unwrap();
    fs::write(dir_b.join("config.yaml"), &yaml).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

    assert_eq!(registry.len(), 1);

    let (_, config) = registry.get_by_name("Issue").expect("Should have Issue");
    assert_eq!(config.name, "Issue");
}

#[tokio::test]
async fn test_registry_build_empty_centy_dir() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");
    fs::create_dir_all(&centy_dir).await.unwrap();

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[tokio::test]
async fn test_registry_build_no_centy_dir() {
    let temp = tempdir().expect("Should create temp dir");

    let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
    assert!(registry.is_empty());
}
