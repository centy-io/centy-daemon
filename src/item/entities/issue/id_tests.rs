use super::*;

#[test]
fn test_is_uuid_valid() {
    assert!(is_uuid("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"));
    assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
}

#[test]
fn test_is_uuid_invalid() {
    assert!(!is_uuid("not-a-uuid"));
    assert!(!is_uuid("0001"));
    assert!(!is_uuid(""));
    assert!(!is_uuid("a3f2b1c9-4d5e-6f7a-8b9c")); // incomplete
}

#[test]
fn test_is_legacy_number_valid() {
    assert!(is_legacy_number("0001"));
    assert!(is_legacy_number("0042"));
    assert!(is_legacy_number("9999"));
}

#[test]
fn test_is_legacy_number_invalid() {
    assert!(!is_legacy_number("001")); // too short
    assert!(!is_legacy_number("00001")); // too long
    assert!(!is_legacy_number("abcd")); // not digits
    assert!(!is_legacy_number("")); // empty
}

#[test]
fn test_is_valid_issue_folder() {
    // Valid UUIDs
    assert!(is_valid_issue_folder(
        "a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"
    ));
    // Valid legacy numbers
    assert!(is_valid_issue_folder("0001"));
    assert!(is_valid_issue_folder("0042"));
    // Invalid
    assert!(!is_valid_issue_folder("random-folder"));
    assert!(!is_valid_issue_folder(".DS_Store"));
}

#[test]
fn test_generate_issue_id() {
    let id = generate_issue_id();
    assert!(is_uuid(&id));
    // Ensure uniqueness
    let id2 = generate_issue_id();
    assert_ne!(id, id2);
}

#[test]
fn test_short_id() {
    assert_eq!(short_id("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"), "a3f2b1c9");
    assert_eq!(short_id("0001"), "0001");
    assert_eq!(short_id("abc"), "abc");
}

#[test]
fn test_is_valid_issue_file() {
    // Valid UUID.md files
    assert!(is_valid_issue_file(
        "a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c.md"
    ));
    assert!(is_valid_issue_file(
        "550e8400-e29b-41d4-a716-446655440000.md"
    ));
    // Invalid files
    assert!(!is_valid_issue_file("0001.md")); // Legacy number not valid for new format
    assert!(!is_valid_issue_file("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c")); // No .md extension
    assert!(!is_valid_issue_file("random.md")); // Not a UUID
    assert!(!is_valid_issue_file("README.md")); // Not a UUID
}

#[test]
fn test_issue_id_from_filename() {
    assert_eq!(
        issue_id_from_filename("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c.md"),
        Some("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c")
    );
    assert_eq!(issue_id_from_filename("0001.md"), None);
    assert_eq!(issue_id_from_filename("random.md"), None);
    assert_eq!(issue_id_from_filename("no-extension"), None);
}
