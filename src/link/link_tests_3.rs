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
fn test_builtin_link_types_symmetry() {
    for (name, inverse) in BUILTIN_LINK_TYPES {
        let has_inverse = BUILTIN_LINK_TYPES.iter().any(|(n, _)| *n == *inverse);
        assert!(
            has_inverse,
            "Inverse '{inverse}' of '{name}' not found in BUILTIN_LINK_TYPES"
        );
    }
}

#[test]
fn test_is_valid_link_type_all_builtins() {
    let custom: Vec<CustomLinkTypeDefinition> = vec![];
    for (name, _) in BUILTIN_LINK_TYPES {
        assert!(is_valid_link_type(name, &custom), "{name} should be valid");
    }
}

#[test]
fn test_get_inverse_all_builtins() {
    let custom: Vec<CustomLinkTypeDefinition> = vec![];
    for (name, expected_inverse) in BUILTIN_LINK_TYPES {
        let inverse = get_inverse_link_type(name, &custom);
        assert_eq!(
            inverse.as_deref(),
            Some(*expected_inverse),
            "Inverse of '{name}' should be '{expected_inverse}'"
        );
    }
}
