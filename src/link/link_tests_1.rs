use super::*;
use std::str::FromStr;

#[test]
fn test_target_type_from_str() {
    assert_eq!(TargetType::from_str("issue").ok(), Some(TargetType::issue()));
    assert_eq!(TargetType::from_str("doc").ok(), Some(TargetType::new("doc")));
    assert_eq!(TargetType::from_str("ISSUE").ok(), Some(TargetType::issue()));
}

#[test]
fn test_target_type_folder_name() {
    assert_eq!(TargetType::issue().folder_name(), "issues");
    assert_eq!(TargetType::new("doc").folder_name(), "docs");
}

#[test]
fn test_get_inverse_builtin() {
    let custom: Vec<CustomLinkTypeDefinition> = vec![];
    assert_eq!(
        get_inverse_link_type("blocks", &custom),
        Some("blocked-by".to_string())
    );
    assert_eq!(
        get_inverse_link_type("blocked-by", &custom),
        Some("blocks".to_string())
    );
    assert_eq!(
        get_inverse_link_type("parent-of", &custom),
        Some("child-of".to_string())
    );
}

#[test]
fn test_get_inverse_custom() {
    let custom = vec![CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        inverse: "dependency-of".to_string(),
        description: None,
    }];
    assert_eq!(
        get_inverse_link_type("depends-on", &custom),
        Some("dependency-of".to_string())
    );
    assert_eq!(
        get_inverse_link_type("dependency-of", &custom),
        Some("depends-on".to_string())
    );
}

#[test]
fn test_get_inverse_unknown() {
    let custom: Vec<CustomLinkTypeDefinition> = vec![];
    assert_eq!(get_inverse_link_type("unknown-type", &custom), None);
}

#[test]
fn test_is_valid_link_type() {
    let custom = vec![CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        inverse: "dependency-of".to_string(),
        description: None,
    }];
    assert!(is_valid_link_type("blocks", &custom));
    assert!(is_valid_link_type("depends-on", &custom));
    assert!(is_valid_link_type("dependency-of", &custom));
    assert!(!is_valid_link_type("invalid-type", &custom));
}

#[test]
fn test_link_serialization() {
    let link = Link::new(
        "uuid-123".to_string(),
        TargetType::issue(),
        "blocks".to_string(),
    );
    let json = serde_json::to_string(&link).unwrap();
    assert!(json.contains("\"targetId\":\"uuid-123\""));
    assert!(json.contains("\"targetType\":\"issue\""));
    assert!(json.contains("\"linkType\":\"blocks\""));
}
