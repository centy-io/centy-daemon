use super::*;

#[test]
fn test_sanitize_filename_valid() {
    assert!(sanitize_filename("test.png").is_ok());
    assert!(sanitize_filename("my-image_01.jpg").is_ok());
    assert!(sanitize_filename("screenshot 2024.png").is_ok());
}

#[test]
fn test_sanitize_filename_invalid() {
    assert!(sanitize_filename("").is_err());
    assert!(sanitize_filename("../test.png").is_err());
    assert!(sanitize_filename("path/to/file.png").is_err());
    assert!(sanitize_filename(".hidden").is_err());
    assert!(sanitize_filename(&"a".repeat(300)).is_err());
}

#[test]
fn test_get_mime_type() {
    assert_eq!(get_mime_type("test.png"), Some("image/png".to_string()));
    assert_eq!(get_mime_type("test.PNG"), Some("image/png".to_string()));
    assert_eq!(get_mime_type("test.jpg"), Some("image/jpeg".to_string()));
    assert_eq!(get_mime_type("test.jpeg"), Some("image/jpeg".to_string()));
    assert_eq!(get_mime_type("test.mp4"), Some("video/mp4".to_string()));
    assert_eq!(get_mime_type("test.webm"), Some("video/webm".to_string()));
    assert_eq!(get_mime_type("test.txt"), None);
    assert_eq!(get_mime_type("test"), None);
}

#[test]
fn test_compute_binary_hash() {
    let data = b"hello world";
    let hash = compute_binary_hash(data);
    assert_eq!(
        hash,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_asset_scope_default() {
    let scope: AssetScope = Default::default();
    assert_eq!(scope, AssetScope::IssueSpecific);
}
