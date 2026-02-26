use super::*;
use mdstore::IdStrategy;

// ─── Default config tests ─────────────────────────────────────────────────

#[test]
fn test_default_issue_config_maps_fields() {
    let mut config = CentyConfig::default();
    config.allowed_states = vec![
        "open".to_string(),
        "in-progress".to_string(),
        "closed".to_string(),
    ];
    config.priority_levels = 5;

    let issue = default_issue_config(&config);

    assert_eq!(issue.name, "Issue");
    assert_eq!(issue.icon, Some("clipboard".to_string()));
    assert_eq!(issue.identifier, IdStrategy::Uuid);
    assert_eq!(issue.statuses, config.allowed_states);
    assert_eq!(issue.default_status, Some("open".to_string()));
    assert_eq!(issue.priority_levels, Some(5));
    assert_eq!(issue.template, Some("template.md".to_string()));
    assert!(issue.features.display_number);
    assert!(issue.features.status);
    assert!(issue.features.priority);
    assert!(issue.features.soft_delete);
    assert!(issue.features.assets);
    assert!(issue.features.org_sync);
    assert!(issue.features.move_item);
    assert!(issue.features.duplicate);
}

#[test]
fn test_default_doc_config() {
    let doc = default_doc_config();

    assert_eq!(doc.name, "Doc");
    assert_eq!(doc.icon, Some("document".to_string()));
    assert_eq!(doc.identifier, IdStrategy::Slug);
    assert!(doc.statuses.is_empty());
    assert!(doc.default_status.is_none());
    assert!(doc.priority_levels.is_none());
    assert!(doc.custom_fields.is_empty());
    assert!(doc.template.is_none());
    assert!(!doc.features.display_number);
    assert!(!doc.features.status);
    assert!(!doc.features.priority);
    assert!(!doc.features.soft_delete);
    assert!(!doc.features.assets);
    assert!(doc.features.org_sync);
    assert!(doc.features.move_item);
    assert!(doc.features.duplicate);
}

#[test]
fn test_default_archived_config() {
    let archived = default_archived_config();

    assert_eq!(archived.name, "Archived");
    assert!(archived.icon.is_none());
    assert_eq!(archived.identifier, IdStrategy::Uuid);
    assert!(archived.statuses.is_empty());
    assert!(archived.default_status.is_none());
    assert!(archived.priority_levels.is_none());
    assert!(archived.template.is_none());
    assert_eq!(archived.custom_fields.len(), 1);
    assert_eq!(archived.custom_fields[0].name, "original_item_type");
    assert_eq!(archived.custom_fields[0].field_type, "string");
    assert!(!archived.features.display_number);
    assert!(!archived.features.status);
    assert!(!archived.features.priority);
    assert!(!archived.features.soft_delete);
    assert!(archived.features.assets);
    assert!(archived.features.org_sync);
    assert!(archived.features.move_item);
    assert!(!archived.features.duplicate);
}

