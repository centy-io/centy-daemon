use super::*;
#[test]
fn test_create_link_options_debug() {
    let opts = CreateLinkOptions {
        source_id: "abc".to_string(),
        source_type: TargetType::issue(),
        target_id: "def".to_string(),
        target_type: TargetType::new("doc"),
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
        link_id: "some-link-uuid".to_string(),
    };
    let debug = format!("{opts:?}");
    assert!(debug.contains("DeleteLinkOptions"));
    assert!(debug.contains("some-link-uuid"));
}
#[test]
fn test_link_type_info_debug() {
    let info = LinkTypeInfo {
        name: "blocks".to_string(),
        description: None,
        is_builtin: true,
    };
    let debug = format!("{info:?}");
    assert!(debug.contains("LinkTypeInfo"));
    assert!(debug.contains("blocks"));
}
#[test]
fn test_get_available_link_types_multiple_custom() {
    let custom = vec![
        CustomLinkTypeDefinition {
            name: "depends-on".to_string(),
            description: None,
        },
        CustomLinkTypeDefinition {
            name: "follows".to_string(),
            description: Some("Sequence order".to_string()),
        },
    ];
    let types = get_available_link_types(&custom);
    // 8 builtin + 2 custom = 10
    assert_eq!(types.len(), 10);
}
