use super::*;

#[test]
fn test_label_to_priority_2_levels() {
    assert_eq!(label_to_priority("high", 2), Some(1));
    assert_eq!(label_to_priority("low", 2), Some(2));
}

#[test]
fn test_label_to_priority_3_levels() {
    assert_eq!(label_to_priority("high", 3), Some(1));
    assert_eq!(label_to_priority("medium", 3), Some(2));
    assert_eq!(label_to_priority("low", 3), Some(3));
}

#[test]
fn test_label_to_priority_4_levels() {
    assert_eq!(label_to_priority("critical", 4), Some(1));
    assert_eq!(label_to_priority("urgent", 4), Some(1));
    assert_eq!(label_to_priority("high", 4), Some(2));
    assert_eq!(label_to_priority("medium", 4), Some(2)); // default
    assert_eq!(label_to_priority("low", 4), Some(4));
}

#[test]
fn test_label_to_priority_p_notation() {
    assert_eq!(label_to_priority("P1", 5), Some(1));
    assert_eq!(label_to_priority("P2", 5), Some(2));
    assert_eq!(label_to_priority("p3", 5), Some(3));
}

#[test]
fn test_label_to_priority_numeric_string() {
    assert_eq!(label_to_priority("1", 5), Some(1));
    assert_eq!(label_to_priority("3", 5), Some(3));
}

#[test]
fn test_label_to_priority_case_insensitive() {
    assert_eq!(label_to_priority("HIGH", 3), Some(1));
    assert_eq!(label_to_priority("Medium", 3), Some(2));
    assert_eq!(label_to_priority("LOW", 3), Some(3));
}

#[test]
fn test_label_to_priority_unknown() {
    assert_eq!(label_to_priority("unknown", 3), None);
    assert_eq!(label_to_priority("", 3), None);
}

#[test]
fn test_migrate_string_priority() {
    // Known labels
    assert_eq!(migrate_string_priority("high", 3), 1);
    assert_eq!(migrate_string_priority("medium", 3), 2);
    assert_eq!(migrate_string_priority("low", 3), 3);

    // Unknown falls back to default
    assert_eq!(migrate_string_priority("unknown", 3), 2);
    assert_eq!(migrate_string_priority("", 3), 2);
}

#[test]
fn test_migrate_string_priority_numeric() {
    assert_eq!(migrate_string_priority("1", 3), 1);
    assert_eq!(migrate_string_priority("2", 3), 2);
    assert_eq!(migrate_string_priority("P1", 5), 1);
}
