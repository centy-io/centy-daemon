//! Additional tests for `create/helpers.rs` covering additional branches.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::helpers::resolve_priority;
use crate::config::CentyConfig;

// --- resolve_priority edge cases ---

#[test]
fn test_resolve_priority_priority_zero_invalid() {
    let result = resolve_priority(Some(0), None, 3);
    assert!(result.is_err());
}

#[test]
fn test_resolve_priority_at_max_level() {
    let result = resolve_priority(Some(3), None, 3).unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_resolve_priority_exceeds_max_level() {
    let result = resolve_priority(Some(4), None, 3);
    assert!(result.is_err());
}

#[test]
fn test_resolve_priority_levels_1_default_is_1() {
    let result = resolve_priority(None, None, 1).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_resolve_priority_levels_2_default_is_1() {
    let result = resolve_priority(None, None, 2).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_resolve_priority_levels_4_default_is_2() {
    let result = resolve_priority(None, None, 4).unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_resolve_priority_config_no_priority_key() {
    // Config has defaults but no "priority" key -> uses default_priority
    let config = CentyConfig::default();
    let result = resolve_priority(None, Some(&config), 3).unwrap();
    assert_eq!(result, 2);
}
