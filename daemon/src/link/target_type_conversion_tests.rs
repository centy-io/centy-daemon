use super::*;
use std::str::FromStr as _;

#[test]
fn test_target_type_from_str() {
    assert_eq!(
        TargetType::from_str("issue").ok(),
        Some(TargetType::issue())
    );
    assert_eq!(
        TargetType::from_str("doc").ok(),
        Some(TargetType::new("doc"))
    );
    assert_eq!(
        TargetType::from_str("ISSUE").ok(),
        Some(TargetType::issue())
    );
}

#[test]
fn test_target_type_folder_name() {
    assert_eq!(TargetType::issue().folder_name(), "issues");
    assert_eq!(TargetType::new("doc").folder_name(), "docs");
}

#[test]
fn test_is_valid_link_type_builtin() {
    let custom: Vec<CustomLinkTypeDefinition> = vec![];
    assert!(is_valid_link_type("blocks", &custom));
    assert!(is_valid_link_type("blocked-by", &custom));
    assert!(is_valid_link_type("parent-of", &custom));
    assert!(!is_valid_link_type("invalid-type", &custom));
}

#[test]
fn test_is_valid_link_type_custom() {
    let custom = vec![CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        description: None,
    }];
    assert!(is_valid_link_type("depends-on", &custom));
    assert!(!is_valid_link_type("dependency-of", &custom));
    assert!(!is_valid_link_type("invalid-type", &custom));
}
