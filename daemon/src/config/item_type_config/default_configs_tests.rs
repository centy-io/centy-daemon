use super::*;
use crate::config::CentyConfig;
use mdstore::IdStrategy;

// ─── Epic default config tests ────────────────────────────────────────────

#[test]
fn test_default_epic_config_is_statusless() {
    let config = CentyConfig {
        priority_levels: 3,
        ..CentyConfig::default()
    };
    let epic = default_epic_config(&config);

    assert_eq!(epic.name, "Epic");
    assert_eq!(epic.icon, Some("map".to_string()));
    assert_eq!(epic.identifier, IdStrategy::Uuid);
    assert!(epic.statuses.is_empty(), "epics must be statusless");
    assert_eq!(epic.priority_levels, Some(3));
    assert_eq!(epic.template, Some("template.md".to_string()));
    assert!(epic.listed);
    assert!(epic.features.display_number);
    assert!(epic.features.priority);
    assert!(epic.features.soft_delete);
    assert!(epic.features.assets);
    assert!(epic.features.org_sync);
    assert!(epic.features.move_item);
    assert!(epic.features.duplicate);
}

// ─── Default config tests ─────────────────────────────────────────────────

#[test]
fn test_default_issue_config_maps_fields() {
    let config = CentyConfig {
        priority_levels: 5,
        ..CentyConfig::default()
    };
    let issue = default_issue_config(&config);

    assert_eq!(issue.name, "Issue");
    assert_eq!(issue.icon, Some("clipboard".to_string()));
    assert_eq!(issue.identifier, IdStrategy::Uuid);
    assert_eq!(
        issue.statuses,
        vec!["open", "planning", "in-progress", "closed"]
    );
    assert_eq!(issue.statuses.first().map(String::as_str), Some("open"));
    assert_eq!(issue.priority_levels, Some(5));
    assert_eq!(issue.template, Some("template.md".to_string()));
    assert!(issue.features.display_number);
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
    assert!(doc.statuses.is_empty());
    assert!(doc.priority_levels.is_none());
    assert!(doc.custom_fields.is_empty());
    assert!(doc.template.is_none());
    assert!(!doc.features.display_number);
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
    assert!(archived.statuses.is_empty());
    assert!(archived.priority_levels.is_none());
    assert!(archived.template.is_none());
    assert_eq!(archived.custom_fields.len(), 1);
    assert_eq!(archived.custom_fields[0].name, "original_item_type");
    assert_eq!(archived.custom_fields[0].field_type, "string");
    assert!(!archived.features.display_number);
    assert!(!archived.features.priority);
    assert!(!archived.features.soft_delete);
    assert!(archived.features.assets);
    assert!(archived.features.org_sync);
    assert!(archived.features.move_item);
    assert!(!archived.features.duplicate);
}
