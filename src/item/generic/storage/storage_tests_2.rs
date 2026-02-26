use super::*;
use crate::config::item_type_config::default_issue_config;
use crate::config::CentyConfig;
use std::collections::HashMap;
async fn setup_project_2(temp: &std::path::Path) {
    let centy_path = temp.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = manifest::create_manifest();
    manifest::write_manifest(temp, &manifest).await.unwrap();
}
fn issue_type_config_2() -> TypeConfig { TypeConfig::from(&default_issue_config(&CentyConfig::default())) }
#[tokio::test]
async fn test_create_invalid_priority() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_2(temp.path()).await;
    let config = issue_type_config_2();
    let options = CreateOptions { title: "Bad Priority".to_string(), body: String::new(), id: None, status: Some("open".to_string()), priority: Some(99), custom_fields: HashMap::new() };
    let result = generic_create(temp.path(), "issues", &config, options).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(ItemError::InvalidPriority { .. })));
}
#[tokio::test]
async fn test_list_with_filters() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_2(temp.path()).await;
    let config = issue_type_config_2();
    for (title, status) in [("Open 1", "open"), ("Open 2", "open"), ("Closed 1", "closed")] {
        let options = CreateOptions { title: title.to_string(), body: String::new(), id: None, status: Some(status.to_string()), priority: Some(2), custom_fields: HashMap::new() };
        generic_create(temp.path(), "issues", &config, options).await.unwrap();
    }
    let all = generic_list(temp.path(), "issues", Filters::default()).await.unwrap();
    assert_eq!(all.len(), 3);
    let open = generic_list(temp.path(), "issues", Filters::new().with_status("open")).await.unwrap();
    assert_eq!(open.len(), 2);
    let limited = generic_list(temp.path(), "issues", Filters::new().with_limit(1)).await.unwrap();
    assert_eq!(limited.len(), 1);
    let offset = generic_list(temp.path(), "issues", Filters::new().with_offset(2)).await.unwrap();
    assert_eq!(offset.len(), 1);
}
#[tokio::test]
async fn test_update() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_2(temp.path()).await;
    let config = issue_type_config_2();
    let options = CreateOptions { title: "Original Title".to_string(), body: "Original body.".to_string(), id: None, status: Some("open".to_string()), priority: Some(2), custom_fields: HashMap::new() };
    let created = generic_create(temp.path(), "issues", &config, options).await.unwrap();
    let update_options = UpdateOptions { title: Some("Updated Title".to_string()), body: Some("Updated body.".to_string()), status: Some("closed".to_string()), priority: Some(1), custom_fields: HashMap::from([("env".to_string(), serde_json::json!("prod"))]) };
    let updated = generic_update(temp.path(), "issues", &config, &created.id, update_options).await.unwrap();
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.body, "Updated body.");
    assert_eq!(updated.frontmatter.status, Some("closed".to_string()));
    assert_eq!(updated.frontmatter.priority, Some(1));
    assert_eq!(updated.frontmatter.custom_fields.get("env"), Some(&serde_json::json!("prod")));
}
#[tokio::test]
async fn test_update_not_found() {
    let temp = tempfile::tempdir().unwrap();
    setup_project_2(temp.path()).await;
    let config = issue_type_config_2();
    let result = generic_update(temp.path(), "issues", &config, "nonexistent", UpdateOptions::default()).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(ItemError::NotFound(_))));
}
