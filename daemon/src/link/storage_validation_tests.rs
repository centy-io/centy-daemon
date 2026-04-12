//! Tests for link/storage/validation.rs covering all branches.
#![allow(clippy::unwrap_used)]

use super::validation::{validate_link_ids, validate_link_type};

// ─── validate_link_type ──────────────────────────────────────────────────────

#[test]
fn test_validate_link_type_non_empty_ok() {
    assert!(validate_link_type("blocks").is_ok());
}

#[test]
fn test_validate_link_type_empty_err() {
    assert!(validate_link_type("").is_err());
}

// ─── validate_link_ids ───────────────────────────────────────────────────────

#[test]
fn test_validate_link_ids_both_non_empty_ok() {
    assert!(validate_link_ids("source-id", "target-id").is_ok());
}

#[test]
fn test_validate_link_ids_empty_source_err() {
    assert!(validate_link_ids("", "target-id").is_err());
}

#[test]
fn test_validate_link_ids_empty_target_err() {
    assert!(validate_link_ids("source-id", "").is_err());
}
