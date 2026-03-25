use super::*;
#[test]
fn test_get_available_link_types_builtin() {
    let custom: Vec<CustomLinkTypeDefinition> = vec![];
    let types = get_available_link_types(&custom);
    assert_eq!(types.len(), 4);
    assert!(types.iter().all(|t| t.is_builtin));
}
#[test]
fn test_get_available_link_types_with_custom() {
    let custom = vec![CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        inverse: "dependency-of".to_string(),
        description: Some("Dependency relationship".to_string()),
    }];
    let types = get_available_link_types(&custom);
    assert_eq!(types.len(), 5);
    let custom_type = types.iter().find(|t| !t.is_builtin).unwrap();
    assert_eq!(custom_type.name, "depends-on");
    assert_eq!(custom_type.inverse, "dependency-of");
    assert_eq!(
        custom_type.description,
        Some("Dependency relationship".to_string())
    );
}
#[test]
fn test_link_error_invalid_link_type() {
    let err = LinkError::InvalidLinkType("unknown-type".to_string());
    let display = format!("{err}");
    assert!(display.contains("Invalid link type"));
    assert!(display.contains("unknown-type"));
}
#[test]
fn test_link_error_source_not_found() {
    let err = LinkError::SourceNotFound("issue-123".to_string(), TargetType::issue());
    let display = format!("{err}");
    assert!(display.contains("Source entity not found"));
    assert!(display.contains("issue-123"));
}
#[test]
fn test_link_error_target_not_found() {
    let err = LinkError::TargetNotFound("doc-slug".to_string(), TargetType::new("doc"));
    let display = format!("{err}");
    assert!(display.contains("Target entity not found"));
    assert!(display.contains("doc-slug"));
}
#[test]
fn test_link_error_already_exists() {
    let err = LinkError::LinkAlreadyExists;
    assert_eq!(format!("{err}"), "Link already exists");
}
#[test]
fn test_link_error_not_found() {
    let err = LinkError::LinkNotFound;
    assert_eq!(format!("{err}"), "Link not found");
}
#[test]
fn test_link_error_self_link() {
    let err = LinkError::SelfLink;
    assert_eq!(format!("{err}"), "Cannot link entity to itself");
}
#[test]
fn test_link_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err = LinkError::from(io_err);
    assert!(matches!(err, LinkError::IoError(_)));
}
