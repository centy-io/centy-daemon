use super::*;
use crate::config::item_type_config::default_issue_config;
use crate::config::CentyConfig;
use std::collections::HashMap;
async fn setup_project_4(temp: &std::path::Path) {
    let centy_path = temp.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = manifest::create_manifest();
    manifest::write_manifest(temp, &manifest).await.unwrap();
}
fn issue_type_config_4() -> TypeConfig { TypeConfig::from(&default_issue_config(&CentyConfig::default())) }
fn minimal_config_4() -> TypeConfig {
    TypeConfig { name: "Note".to_string(), identifier: mdstore::IdStrategy::Uuid, features: mdstore::TypeFeatures::default(), statuses: Vec::new(), default_status: None, priority_levels: None, custom_fields: Vec::new() }
}
#[tokio::test]
async fn test_cannot_update_deleted_item() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_4(temp.path()).await;
    let config = issue_type_config_4();
    let options = CreateOptions { title: "Will Delete".to_string(), body: String::new(), id: None, status: Some("open".to_string()), priority: Some(2), custom_fields: HashMap::new() };
    let created = generic_create(temp.path(), "issues", &config, options).await.unwrap();
    generic_soft_delete(temp.path(), "issues", &created.id).await.unwrap();
    let result = generic_update(temp.path(), "issues", &config, &created.id, UpdateOptions { title: Some("Fail".to_string()), ..Default::default() }).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(ItemError::IsDeleted(_))));
}
#[tokio::test]
async fn test_already_exists() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_4(temp.path()).await;
    let mut config = minimal_config_4();
    config.identifier = mdstore::IdStrategy::Slug;
    let options = CreateOptions { title: "Same Title".to_string(), body: String::new(), id: None, status: None, priority: None, custom_fields: HashMap::new() };
    generic_create(temp.path(), "notes", &config, options.clone()).await.unwrap();
    let result = generic_create(temp.path(), "notes", &config, options).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(ItemError::AlreadyExists(_))));
}
#[tokio::test]
async fn test_get_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_4(temp.path()).await;
    let result = generic_get(temp.path(), "issues", "nonexistent").await;
    assert!(result.is_err());
    assert!(matches!(result, Err(ItemError::NotFound(_))));
}
#[tokio::test]
async fn test_list_empty() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_4(temp.path()).await;
    let items = generic_list(temp.path(), "issues", Filters::default()).await.unwrap();
    assert!(items.is_empty());
}
#[tokio::test]
async fn test_get_by_display_number_success() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_4(temp.path()).await;
    let config = issue_type_config_4();
    let options1 = CreateOptions { title: "First Issue".to_string(), body: "Body 1".to_string(), id: None, status: Some("open".to_string()), priority: Some(2), custom_fields: HashMap::new() };
    let created1 = generic_create(temp.path(), "issues", &config, options1).await.unwrap();
    assert_eq!(created1.frontmatter.display_number, Some(1));
    let options2 = CreateOptions { title: "Second Issue".to_string(), body: "Body 2".to_string(), id: None, status: Some("open".to_string()), priority: Some(1), custom_fields: HashMap::new() };
    let created2 = generic_create(temp.path(), "issues", &config, options2).await.unwrap();
    assert_eq!(created2.frontmatter.display_number, Some(2));
    let found = generic_get_by_display_number(temp.path(), "issues", &config, 2).await.unwrap();
    assert_eq!(found.title, "Second Issue");
    assert_eq!(found.id, created2.id);
}
#[tokio::test]
async fn test_get_by_display_number_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_4(temp.path()).await;
    let config = issue_type_config_4();
    let options = CreateOptions { title: "Only Issue".to_string(), body: String::new(), id: None, status: Some("open".to_string()), priority: Some(2), custom_fields: HashMap::new() };
    generic_create(temp.path(), "issues", &config, options).await.unwrap();
    let result = generic_get_by_display_number(temp.path(), "issues", &config, 99).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(ItemError::NotFound(_))));
}
#[tokio::test]
async fn test_get_by_display_number_feature_disabled() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_4(temp.path()).await;
    let config = minimal_config_4();
    let result = generic_get_by_display_number(temp.path(), "notes", &config, 1).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(ItemError::FeatureNotEnabled(_))));
}
