use super::*;
use crate::config::item_type_config::default_issue_config;
use crate::config::CentyConfig;
use std::collections::HashMap;
async fn setup_project(temp: &std::path::Path) {
    let centy_path = temp.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = manifest::create_manifest();
    manifest::write_manifest(temp, &manifest).await.unwrap();
}
fn issue_type_config() -> TypeConfig {
    TypeConfig::from(&default_issue_config(&CentyConfig::default()))
}
fn minimal_config() -> TypeConfig {
    TypeConfig {
        name: "Note".to_string(),
        identifier: mdstore::IdStrategy::Uuid,
        features: mdstore::TypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    }
}
#[tokio::test]
async fn test_create_and_get() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let config = issue_type_config();
    let options = CreateOptions { title: "Test Issue".to_string(), body: "This is a test.".to_string(), id: None, status: Some("open".to_string()), priority: Some(2), custom_fields: HashMap::new() };
    let created = generic_create(temp.path(), "issues", &config, options).await.unwrap();
    assert_eq!(created.title, "Test Issue");
    assert_eq!(created.body, "This is a test.");
    assert_eq!(created.frontmatter.display_number, Some(1));
    assert_eq!(created.frontmatter.status, Some("open".to_string()));
    assert_eq!(created.frontmatter.priority, Some(2));
    let fetched = generic_get(temp.path(), "issues", &created.id).await.unwrap();
    assert_eq!(fetched.title, "Test Issue");
    assert_eq!(fetched.frontmatter.display_number, Some(1));
}
#[tokio::test]
async fn test_create_minimal_features() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let config = minimal_config();
    let options = CreateOptions { title: "Simple Note".to_string(), body: "Just a note.".to_string(), id: None, status: None, priority: None, custom_fields: HashMap::new() };
    let created = generic_create(temp.path(), "notes", &config, options).await.unwrap();
    assert!(created.frontmatter.display_number.is_none());
    assert!(created.frontmatter.status.is_none());
    assert!(created.frontmatter.priority.is_none());
}
#[tokio::test]
async fn test_create_slug_id_strategy() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let mut config = minimal_config();
    config.identifier = mdstore::IdStrategy::Slug;
    let options = CreateOptions { title: "Getting Started Guide".to_string(), body: "Welcome!".to_string(), id: None, status: None, priority: None, custom_fields: HashMap::new() };
    let created = generic_create(temp.path(), "docs", &config, options).await.unwrap();
    assert_eq!(created.id, "getting-started-guide");
}
#[tokio::test]
async fn test_create_invalid_status() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    let config = issue_type_config();
    let options = CreateOptions { title: "Bad Status".to_string(), body: String::new(), id: None, status: Some("nonexistent".to_string()), priority: None, custom_fields: HashMap::new() };
    let result = generic_create(temp.path(), "issues", &config, options).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(ItemError::InvalidStatus { .. })));
}
