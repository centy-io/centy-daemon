use super::*;
use crate::registry::organizations::slugify;

#[test]
fn test_slugify_catches_underscore_vs_hyphen() {
    // These two folder names produce the same slug
    assert_eq!(slugify("my_app"), slugify("my-app"));
    assert_eq!(slugify("my_app"), "my-app");
}

#[test]
fn test_extract_project_name() {
    assert_eq!(
        ValidationService::extract_project_name("/path/to/myapp"),
        Some("myapp".to_string())
    );
    assert_eq!(
        ValidationService::extract_project_name("/myapp"),
        Some("myapp".to_string())
    );
    assert_eq!(
        ValidationService::extract_project_name("myapp"),
        Some("myapp".to_string())
    );
}
