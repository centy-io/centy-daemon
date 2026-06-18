use super::*;
#[test]
fn test_project_registry_new() {
    let reg = ProjectRegistry::new();
    assert_eq!(reg.schema_version, CURRENT_SCHEMA_VERSION);
    assert!(!reg.updated_at.is_empty());
    assert!(reg.organizations.is_empty());
    assert!(reg.projects.is_empty());
}
#[test]
fn test_project_registry_default() {
    let reg = ProjectRegistry::default();
    assert_eq!(reg.schema_version, 0);
    assert!(reg.projects.is_empty());
    assert!(reg.organizations.is_empty());
}
#[test]
fn test_project_registry_serialization() {
    let reg = ProjectRegistry::new();
    let json = serde_json::to_string(&reg).expect("Should serialize");
    let deserialized: ProjectRegistry = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.schema_version, CURRENT_SCHEMA_VERSION);
}
#[test]
fn test_current_schema_version() {
    assert_eq!(CURRENT_SCHEMA_VERSION, 2);
}
#[test]
fn test_list_projects_options_default() {
    let opts = ListProjectsOptions::default();
    assert!(!opts.include_stale);
    assert!(!opts.include_uninitialized);
    assert!(!opts.include_archived);
    assert!(opts.organization_slug.is_none());
    assert!(!opts.ungrouped_only);
    assert!(!opts.include_temp);
}
#[test]
fn test_list_projects_options_with_org_filter() {
    let opts = ListProjectsOptions {
        organization_slug: Some("acme"),
        ..Default::default()
    };
    assert_eq!(opts.organization_slug, Some("acme"));
}
#[test]
fn test_project_info_debug() {
    let info = ProjectInfo {
        path: "/home/user/project".to_string(),
        first_accessed: "2024-01-01".to_string(),
        last_accessed: "2024-06-15".to_string(),
        issue_count: 10,
        doc_count: 5,
        initialized: true,
        name: Some("my-project".to_string()),
        is_favorite: true,
        is_archived: false,
        organization_slug: None,
        organization_name: None,
        user_title: None,
        project_title: None,
        project_version: None,
        project_behind: false,
    };
    let debug = format!("{info:?}");
    assert!(debug.contains("ProjectInfo"));
    assert!(debug.contains("my-project"));
}
#[test]
fn test_organization_info_debug() {
    let info = OrganizationInfo {
        slug: "acme".to_string(),
        name: "Acme Corp".to_string(),
        description: Some("desc".to_string()),
        created_at: "2024-01-01".to_string(),
        updated_at: "2024-01-01".to_string(),
        project_count: 3,
    };
    let debug = format!("{info:?}");
    assert!(debug.contains("OrganizationInfo"));
    assert!(debug.contains("Acme Corp"));
    assert!(debug.contains('3'));
}
