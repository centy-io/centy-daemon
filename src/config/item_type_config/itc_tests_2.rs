use super::*;

#[test]
fn test_archived_config_yaml_serialization() {
    let config = default_archived_config();
    let yaml = serde_yaml::to_string(&config).expect("Should serialize");

    assert!(yaml.contains("name: Archived"));
    assert!(yaml.contains("identifier: uuid"));
    assert!(yaml.contains("displayNumber: false"));
    assert!(yaml.contains("status: false"));
    assert!(yaml.contains("priority: false"));
    assert!(yaml.contains("softDelete: false"));
    assert!(yaml.contains("assets: true"));
    assert!(yaml.contains("orgSync: true"));
    assert!(yaml.contains("move: true"));
    assert!(yaml.contains("duplicate: false"));
    assert!(yaml.contains("original_item_type"));
    assert!(!yaml.contains("icon:"));
    assert!(!yaml.contains("template:"));
    assert!(!yaml.contains("statuses:"));
    assert!(!yaml.contains("defaultStatus:"));
    assert!(!yaml.contains("priorityLevels:"));
}

#[test]
fn test_issue_config_yaml_serialization() {
    let config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&config).expect("Should serialize");

    assert!(yaml.contains("name: Issue"));
    assert!(yaml.contains("icon: clipboard"));
    assert!(yaml.contains("identifier: uuid"));
    assert!(yaml.contains("displayNumber: true"));
    assert!(yaml.contains("softDelete: true"));
    assert!(yaml.contains("move: true"));
    assert!(yaml.contains("defaultStatus: open"));
    assert!(yaml.contains("template: template.md"));
}

#[test]
fn test_doc_config_yaml_serialization() {
    let config = default_doc_config();
    let yaml = serde_yaml::to_string(&config).expect("Should serialize");

    assert!(yaml.contains("name: Doc"));
    assert!(yaml.contains("icon: document"));
    assert!(yaml.contains("identifier: slug"));
    assert!(yaml.contains("displayNumber: false"));
    assert!(yaml.contains("softDelete: false"));
    assert!(!yaml.contains("statuses"));
    assert!(!yaml.contains("defaultStatus"));
    assert!(!yaml.contains("priorityLevels"));
    assert!(!yaml.contains("template:"));
}

#[test]
fn test_item_type_config_yaml_roundtrip() {
    let config = default_issue_config(&CentyConfig::default());
    let yaml = serde_yaml::to_string(&config).expect("Should serialize");
    let deserialized: ItemTypeConfig = serde_yaml::from_str(&yaml).expect("Should deserialize");

    assert_eq!(deserialized.name, "Issue");
    assert_eq!(deserialized.icon, Some("clipboard".to_string()));
    assert_eq!(deserialized.statuses.len(), config.statuses.len());
    assert_eq!(deserialized.default_status, config.default_status);
    assert_eq!(deserialized.priority_levels, config.priority_levels);
    assert_eq!(deserialized.features.soft_delete, config.features.soft_delete);
    assert_eq!(deserialized.template, config.template);
}

// ─── Backward-compat deserialization ─────────────────────────────────────

#[test]
fn test_legacy_yaml_without_new_fields_deserializes() {
    let yaml = "name: Issue\nidentifier: uuid\nfeatures:\n  displayNumber: true\n  status: true\n  priority: true\n  assets: true\n  orgSync: true\n  move: true\n  duplicate: true\nstatuses:\n  - open\n  - closed\ndefaultStatus: open\npriorityLevels: 3\n";
    let config: ItemTypeConfig = serde_yaml::from_str(yaml).expect("Should deserialize");

    assert_eq!(config.name, "Issue");
    assert!(config.icon.is_none());
    assert!(config.template.is_none());
    assert!(!config.features.soft_delete);
}

#[test]
fn test_yaml_with_icon_and_template() {
    let yaml = "name: Task\nicon: tasks\nidentifier: uuid\nfeatures:\n  displayNumber: true\n  status: true\n  priority: false\n  softDelete: false\n  assets: false\n  orgSync: false\n  move: true\n  duplicate: true\nstatuses:\n  - open\n  - closed\ndefaultStatus: open\ntemplate: task-template.md\n";
    let config: ItemTypeConfig = serde_yaml::from_str(yaml).expect("Should deserialize");

    assert_eq!(config.name, "Task");
    assert_eq!(config.icon, Some("tasks".to_string()));
    assert_eq!(config.template, Some("task-template.md".to_string()));
}

