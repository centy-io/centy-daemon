use super::*;

#[test]
fn test_validate_priority_valid() {
    assert!(validate_priority(1, 3).is_ok());
    assert!(validate_priority(2, 3).is_ok());
    assert!(validate_priority(3, 3).is_ok());
}

#[test]
fn test_validate_priority_out_of_range() {
    assert!(validate_priority(0, 3).is_err());
    assert!(validate_priority(4, 3).is_err());
    assert!(validate_priority(10, 3).is_err());
}

#[test]
fn test_validate_priority_single_level() {
    assert!(validate_priority(1, 1).is_ok());
    assert!(validate_priority(0, 1).is_err());
    assert!(validate_priority(2, 1).is_err());
}

#[test]
fn test_default_priority() {
    assert_eq!(default_priority(1), 1);
    assert_eq!(default_priority(2), 1);
    assert_eq!(default_priority(3), 2);
    assert_eq!(default_priority(4), 2);
    assert_eq!(default_priority(5), 3);
    assert_eq!(default_priority(6), 3);
    assert_eq!(default_priority(10), 5);
}

#[test]
fn test_default_priority_edge_case() {
    // Zero should return 1 (safe default)
    assert_eq!(default_priority(0), 1);
}

#[test]
fn test_priority_label_1_level() {
    assert_eq!(priority_label(1, 1), "normal");
}

#[test]
fn test_priority_label_2_levels() {
    assert_eq!(priority_label(1, 2), "high");
    assert_eq!(priority_label(2, 2), "low");
}

#[test]
fn test_priority_label_3_levels() {
    assert_eq!(priority_label(1, 3), "high");
    assert_eq!(priority_label(2, 3), "medium");
    assert_eq!(priority_label(3, 3), "low");
}

#[test]
fn test_priority_label_4_levels() {
    assert_eq!(priority_label(1, 4), "critical");
    assert_eq!(priority_label(2, 4), "high");
    assert_eq!(priority_label(3, 4), "medium");
    assert_eq!(priority_label(4, 4), "low");
}

#[test]
fn test_priority_label_5_plus_levels() {
    assert_eq!(priority_label(1, 5), "P1");
    assert_eq!(priority_label(2, 5), "P2");
    assert_eq!(priority_label(3, 5), "P3");
    assert_eq!(priority_label(4, 5), "P4");
    assert_eq!(priority_label(5, 5), "P5");
}
