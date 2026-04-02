#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::helpers::resolve_priority;
use crate::config::CentyConfig;

// --- resolve_priority tests ---

#[test]
fn test_resolve_priority_none_no_config() {
    // No priority provided, no config -> default_priority(3) == 2
    let result = resolve_priority(None, None, 3).unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_resolve_priority_with_explicit_value() {
    let result = resolve_priority(Some(1), None, 3).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_resolve_priority_invalid_exceeds_levels() {
    let result = resolve_priority(Some(5), None, 3);
    assert!(result.is_err());
}

#[test]
fn test_resolve_priority_from_config_default() {
    let mut config = CentyConfig::default();
    config
        .defaults
        .insert("priority".to_string(), "1".to_string());
    let result = resolve_priority(None, Some(&config), 3).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_resolve_priority_config_default_invalid_string_falls_back() {
    let mut config = CentyConfig::default();
    config
        .defaults
        .insert("priority".to_string(), "not-a-number".to_string());
    // Invalid string can't parse, falls back to default_priority(3) == 2
    let result = resolve_priority(None, Some(&config), 3).unwrap();
    assert_eq!(result, 2);
}
