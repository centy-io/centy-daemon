use super::*;

#[test]
fn test_empty_filter_returns_defaults() {
    let f = build_filters_from_mql("", 0, 0);
    assert!(f.statuses.is_none());
    assert!(f.priority.is_none());
    assert!(!f.include_deleted);
    assert!(f.limit.is_none());
    assert!(f.offset.is_none());
}

#[test]
fn test_pagination_applied() {
    let f = build_filters_from_mql("", 10, 5);
    assert_eq!(f.limit, Some(10));
    assert_eq!(f.offset, Some(5));
}

#[test]
fn test_status_exact_match() {
    let f = build_filters_from_mql(r#"{"status":"open"}"#, 0, 0);
    assert_eq!(f.statuses, Some(vec!["open".to_string()]));
}

#[test]
fn test_status_in_operator() {
    let f = build_filters_from_mql(r#"{"status":{"$in":["open","in-progress"]}}"#, 0, 0);
    assert_eq!(
        f.statuses,
        Some(vec!["open".to_string(), "in-progress".to_string()])
    );
}

#[test]
fn test_priority_exact() {
    let f = build_filters_from_mql(r#"{"priority":1}"#, 0, 0);
    assert_eq!(f.priority, Some(1));
}

#[test]
fn test_priority_lte() {
    let f = build_filters_from_mql(r#"{"priority":{"$lte":2}}"#, 0, 0);
    assert_eq!(f.priority_lte, Some(2));
}

#[test]
fn test_priority_gte() {
    let f = build_filters_from_mql(r#"{"priority":{"$gte":1}}"#, 0, 0);
    assert_eq!(f.priority_gte, Some(1));
}

#[test]
fn test_deleted_at_exists() {
    let f = build_filters_from_mql(r#"{"deletedAt":{"$exists":true}}"#, 0, 0);
    assert!(f.include_deleted);
}

#[test]
fn test_invalid_json_returns_defaults() {
    let f = build_filters_from_mql("not-json", 0, 0);
    assert!(f.statuses.is_none());
    assert!(f.priority.is_none());
}

#[test]
fn test_combined_filter() {
    let f = build_filters_from_mql(
        r#"{"status":{"$in":["open","in-progress"]},"priority":{"$lte":2}}"#,
        20,
        0,
    );
    assert_eq!(
        f.statuses,
        Some(vec!["open".to_string(), "in-progress".to_string()])
    );
    assert_eq!(f.priority_lte, Some(2));
    assert_eq!(f.limit, Some(20));
}
