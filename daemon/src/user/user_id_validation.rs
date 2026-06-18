use super::*;
#[test]
fn test_slugify() {
    assert_eq!(slugify("John Doe"), "john-doe");
    assert_eq!(slugify("Jane Smith"), "jane-smith");
    assert_eq!(slugify("Bob"), "bob");
    assert_eq!(slugify("Test  User"), "test-user");
    assert_eq!(slugify("  leading"), "leading");
    assert_eq!(slugify("trailing  "), "trailing");
    assert_eq!(slugify("UPPERCASE NAME"), "uppercase-name");
    assert_eq!(slugify("user123"), "user123");
}
#[test]
fn test_slugify_special_chars() {
    // cspell:ignore malley
    assert_eq!(slugify("O'Malley"), "o-malley");
    assert_eq!(slugify("user@email.com"), "user-email-com");
    assert_eq!(slugify("first.last"), "first-last");
}
#[test]
fn test_validate_user_id() {
    assert!(validate_user_id("john-doe").is_ok());
    assert!(validate_user_id("jane-smith-123").is_ok());
    assert!(validate_user_id("bob").is_ok());
    assert!(validate_user_id("").is_err());
    assert!(validate_user_id("-start-with-hyphen").is_err());
    assert!(validate_user_id("end-with-hyphen-").is_err());
    assert!(validate_user_id("UPPERCASE").is_err());
    assert!(validate_user_id("has spaces").is_err());
    assert!(validate_user_id("has_underscore").is_err());
}
#[test]
fn test_validate_user_id_error_messages() {
    let err = validate_user_id("").unwrap_err();
    assert!(format!("{err}").contains("empty"));
    let err = validate_user_id("UPPER").unwrap_err();
    assert!(format!("{err}").contains("lowercase"));
    let err = validate_user_id("-start").unwrap_err();
    assert!(format!("{err}").contains("hyphen"));
}
#[test]
fn test_user_serialization() {
    let user = User {
        id: "john-doe".to_string(),
        name: "John Doe".to_string(),
        email: Some("john@example.com".to_string()),
        git_usernames: vec!["johndoe".to_string()],
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-06-15T12:00:00Z".to_string(),
        deleted_at: None,
    };
    let json = serde_json::to_string(&user).expect("Should serialize");
    let deserialized: User = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.id, "john-doe");
    assert_eq!(deserialized.name, "John Doe");
    assert_eq!(deserialized.email, Some("john@example.com".to_string()));
    assert_eq!(deserialized.git_usernames, vec!["johndoe"]);
}
#[test]
fn test_user_serialization_camel_case() {
    let user = User {
        id: "test".to_string(),
        name: "Test".to_string(),
        email: None,
        git_usernames: vec!["gh-user".to_string()],
        created_at: "2024-01-01".to_string(),
        updated_at: "2024-01-01".to_string(),
        deleted_at: None,
    };
    let json = serde_json::to_string(&user).expect("Should serialize");
    assert!(json.contains("createdAt"));
    assert!(json.contains("updatedAt"));
    assert!(json.contains("gitUsernames"));
    assert!(!json.contains("created_at"));
    assert!(!json.contains("git_usernames"));
}
#[test]
fn test_user_skip_serializing_empty_fields() {
    let user = User {
        id: "test".to_string(),
        name: "Test".to_string(),
        email: None,
        git_usernames: vec![],
        created_at: "2024-01-01".to_string(),
        updated_at: "2024-01-01".to_string(),
        deleted_at: None,
    };
    let json = serde_json::to_string(&user).expect("Should serialize");
    assert!(!json.contains("email"));
    assert!(!json.contains("gitUsernames"));
    assert!(!json.contains("deletedAt"));
}
