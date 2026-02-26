use super::*;
#[test]
fn test_create_link_options_debug() {
    let opts = CreateLinkOptions {
        source_id: "abc".to_string(),
        source_type: TargetType::Issue,
        target_id: "def".to_string(),
        target_type: TargetType::Doc,
        link_type: "blocks".to_string(),
    };
    let debug = format!("{opts:?}");
    assert!(debug.contains("CreateLinkOptions"));
    assert!(debug.contains("abc"));
    assert!(debug.contains("def"));
}
#[test]
fn test_delete_link_options_debug() {
    let opts = DeleteLinkOptions {
        source_id: "abc".to_string(),
        source_type: TargetType::Issue,
        target_id: "def".to_string(),
        target_type: TargetType::Doc,
        link_type: Some("blocks".to_string()),
    };
    let debug = format!("{opts:?}");
    assert!(debug.contains("DeleteLinkOptions"));
}
#[test]
fn test_delete_link_options_without_type() {
    let opts = DeleteLinkOptions {
        source_id: "abc".to_string(),
        source_type: TargetType::Issue,
        target_id: "def".to_string(),
        target_type: TargetType::Doc,
        link_type: None,
    };
    assert!(opts.link_type.is_none());
}
#[test]
fn test_link_type_info_debug() {
    let info = LinkTypeInfo {
        name: "blocks".to_string(),
        inverse: "blocked-by".to_string(),
        description: None,
        is_builtin: true,
    };
    let debug = format!("{info:?}");
    assert!(debug.contains("LinkTypeInfo"));
    assert!(debug.contains("blocks"));
}
#[test]
fn test_link_type_info_clone() {
    let info = LinkTypeInfo {
        name: "blocks".to_string(),
        inverse: "blocked-by".to_string(),
        description: Some("Blocking relationship".to_string()),
        is_builtin: true,
    };
    let cloned = info.clone();
    assert_eq!(cloned.name, "blocks");
    assert_eq!(cloned.inverse, "blocked-by");
    assert!(cloned.is_builtin);
}
#[test]
fn test_get_available_link_types_multiple_custom() {
    let custom = vec![
        CustomLinkTypeDefinition {
            name: "depends-on".to_string(),
            inverse: "dependency-of".to_string(),
            description: None,
        },
        CustomLinkTypeDefinition {
            name: "follows".to_string(),
            inverse: "preceded-by".to_string(),
            description: Some("Sequence order".to_string()),
        },
    ];
    let types = get_available_link_types(&custom);
    assert_eq!(types.len(), 6);
}
