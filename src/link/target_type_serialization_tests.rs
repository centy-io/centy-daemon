use super::*;

#[test]
fn test_target_type_as_str() {
    assert_eq!(TargetType::issue().as_str(), "issue");
    assert_eq!(TargetType::new("doc").as_str(), "doc");
}

#[test]
fn test_target_type_display() {
    assert_eq!(format!("{}", TargetType::issue()), "issue");
    assert_eq!(format!("{}", TargetType::new("doc")), "doc");
}

#[test]
fn test_target_type_serialization() {
    let json = serde_json::to_string(&TargetType::issue()).unwrap();
    assert_eq!(json, "\"issue\"");

    let json = serde_json::to_string(&TargetType::new("doc")).unwrap();
    assert_eq!(json, "\"doc\"");
}

#[test]
fn test_target_type_deserialization() {
    let tt: TargetType = serde_json::from_str("\"issue\"").unwrap();
    assert_eq!(tt, TargetType::issue());

    let tt: TargetType = serde_json::from_str("\"doc\"").unwrap();
    assert_eq!(tt, TargetType::new("doc"));
}

#[test]
fn test_builtin_link_types_all_valid() {
    let custom: Vec<CustomLinkTypeDefinition> = vec![];
    for &name in BUILTIN_LINK_TYPES {
        assert!(is_valid_link_type(name, &custom), "{name} should be valid");
    }
}

#[test]
fn test_builtin_link_types_count() {
    assert_eq!(BUILTIN_LINK_TYPES.len(), 8);
}
