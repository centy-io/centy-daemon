use super::*;
use mdstore::{CustomFieldDef, IdStrategy};
use tempfile::tempdir;
use tokio::fs;

// ─── mdstore conversion ───────────────────────────────────────────────────

#[test]
fn test_type_config_from_item_type_config() {
    use mdstore::TypeConfig;
    let item_config = default_issue_config(&CentyConfig::default());
    let type_config = TypeConfig::from(&item_config);

    assert_eq!(type_config.name, "Issue");
    assert_eq!(type_config.identifier, IdStrategy::Uuid);
    assert_eq!(type_config.statuses, item_config.statuses);
    assert_eq!(type_config.default_status, item_config.default_status);
    assert_eq!(type_config.priority_levels, item_config.priority_levels);
    assert!(type_config.features.display_number);
    assert!(type_config.features.status);
    assert!(type_config.features.priority);
    assert!(type_config.features.assets);
    assert!(type_config.features.org_sync);
    assert!(type_config.features.move_item);
    assert!(type_config.features.duplicate);
}

#[test]
fn test_type_config_conversion_drops_new_fields() {
    use mdstore::TypeConfig;
    let item_config = ItemTypeConfig {
        name: "Task".to_string(),
        icon: Some("tasks".to_string()),
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures {
            soft_delete: true,
            ..ItemTypeFeatures::default()
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
        template: Some("task.md".to_string()),
    };
    let type_config = TypeConfig::from(&item_config);
    assert_eq!(type_config.name, "Task");
}

// ─── File I/O tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_read_item_type_config_nonexistent() {
    let temp = tempdir().expect("Should create temp dir");
    let result = read_item_type_config(temp.path(), "issues")
        .await
        .expect("Should not error");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_write_and_read_item_type_config() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy").join("issues");
    fs::create_dir_all(&centy_dir)
        .await
        .expect("Should create dir");

    let config = default_issue_config(&CentyConfig::default());
    write_item_type_config(temp.path(), "issues", &config)
        .await
        .expect("Should write");

    let read = read_item_type_config(temp.path(), "issues")
        .await
        .expect("Should read")
        .expect("Should exist");

    assert_eq!(read.name, "Issue");
    assert_eq!(read.icon, Some("clipboard".to_string()));
    assert_eq!(read.statuses, config.statuses);
    assert_eq!(read.features.soft_delete, config.features.soft_delete);
    assert_eq!(read.template, config.template);
}

#[test]
fn test_issue_config_custom_fields_mapped() {
    let mut config = CentyConfig::default();
    config.custom_fields = vec![CustomFieldDef {
        name: "environment".to_string(),
        field_type: "enum".to_string(),
        required: true,
        default_value: Some("dev".to_string()),
        enum_values: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
    }];

    let issue = default_issue_config(&config);
    assert_eq!(issue.custom_fields.len(), 1);
    assert_eq!(issue.custom_fields[0].name, "environment");
}
