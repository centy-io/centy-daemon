use super::*;
use crate::config::item_type_config::default_issue_config;
use crate::config::CentyConfig;
use std::collections::HashMap;
async fn setup_project_3(temp: &std::path::Path) {
    let centy_path = temp.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = manifest::create_manifest();
    manifest::write_manifest(temp, &manifest).await.unwrap();
}
fn issue_type_config_3() -> TypeConfig { TypeConfig::from(&default_issue_config(&CentyConfig::default())) }
#[tokio::test]
async fn test_soft_delete_and_restore() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_3(temp.path()).await;
    let config = issue_type_config_3();
    let options = CreateOptions { title: "To Delete".to_string(), body: String::new(), id: None, status: Some("open".to_string()), priority: Some(2), custom_fields: HashMap::new() };
    let created = generic_create(temp.path(), "issues", &config, options).await.unwrap();
    generic_soft_delete(temp.path(), "issues", &created.id).await.unwrap();
    let items = generic_list(temp.path(), "issues", Filters::default()).await.unwrap();
    assert!(items.is_empty());
    let items = generic_list(temp.path(), "issues", Filters::new().include_deleted()).await.unwrap();
    assert_eq!(items.len(), 1);
    assert!(items.first().unwrap().frontmatter.deleted_at.is_some());
    generic_restore(temp.path(), "issues", &created.id).await.unwrap();
    let items = generic_list(temp.path(), "issues", Filters::default()).await.unwrap();
    assert_eq!(items.len(), 1);
    assert!(items.first().unwrap().frontmatter.deleted_at.is_none());
}
#[tokio::test]
async fn test_hard_delete() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_3(temp.path()).await;
    let config = issue_type_config_3();
    let options = CreateOptions { title: "To Hard Delete".to_string(), body: String::new(), id: None, status: Some("open".to_string()), priority: Some(2), custom_fields: HashMap::new() };
    let created = generic_create(temp.path(), "issues", &config, options).await.unwrap();
    generic_delete(temp.path(), "issues", &config, &created.id, true).await.unwrap();
    let result = generic_get(temp.path(), "issues", &created.id).await;
    assert!(result.is_err());
}
#[tokio::test]
async fn test_display_number_auto_increment() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_3(temp.path()).await;
    let config = issue_type_config_3();
    for i in 1..=3u32 {
        let options = CreateOptions { title: format!("Issue {i}"), body: String::new(), id: None, status: Some("open".to_string()), priority: Some(2), custom_fields: HashMap::new() };
        let created = generic_create(temp.path(), "issues", &config, options).await.unwrap();
        assert_eq!(created.frontmatter.display_number, Some(i));
    }
}
#[tokio::test]
async fn test_update_preserves_fields() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_3(temp.path()).await;
    let config = issue_type_config_3();
    let options = CreateOptions { title: "Keep Fields".to_string(), body: "Original body.".to_string(), id: None, status: Some("open".to_string()), priority: Some(1), custom_fields: HashMap::from([("key".to_string(), serde_json::json!("value"))]) };
    let created = generic_create(temp.path(), "issues", &config, options).await.unwrap();
    let updated = generic_update(temp.path(), "issues", &config, &created.id, UpdateOptions { title: Some("New Title".to_string()), ..Default::default() }).await.unwrap();
    assert_eq!(updated.title, "New Title");
    assert_eq!(updated.body, "Original body.");
    assert_eq!(updated.frontmatter.status, Some("open".to_string()));
    assert_eq!(updated.frontmatter.priority, Some(1));
    assert_eq!(updated.frontmatter.custom_fields.get("key"), Some(&serde_json::json!("value")));
}
