use super::*;

#[test]
fn test_is_version_behind_older() {
    assert!(is_version_behind("0.7.0", "0.8.0"));
}

#[test]
fn test_is_version_behind_same() {
    assert!(!is_version_behind("0.8.0", "0.8.0"));
}

#[test]
fn test_is_version_behind_newer() {
    assert!(!is_version_behind("0.9.0", "0.8.0"));
}

#[test]
fn test_is_version_behind_invalid() {
    assert!(!is_version_behind("invalid", "0.8.0"));
    assert!(!is_version_behind("0.8.0", "invalid"));
}
