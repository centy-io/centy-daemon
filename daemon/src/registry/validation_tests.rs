use super::*;

#[test]
fn test_normalize_name() {
    assert_eq!(ValidationService::normalize_name("MyApp"), "myapp");
    assert_eq!(ValidationService::normalize_name("  MyApp  "), "myapp");
    assert_eq!(ValidationService::normalize_name("MYAPP"), "myapp");
    assert_eq!(ValidationService::normalize_name("myapp"), "myapp");
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
