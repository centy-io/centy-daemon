use super::*;

#[test]
fn test_parse_valid_pattern() {
    let p = ParsedPattern::parse("pre:issue:create").unwrap();
    assert_eq!(p.phase, PatternSegment::Exact("pre".to_string()));
    assert_eq!(p.item_type, PatternSegment::Exact("issue".to_string()));
    assert_eq!(p.operation, PatternSegment::Exact("create".to_string()));
}

#[test]
fn test_parse_wildcard_pattern() {
    let p = ParsedPattern::parse("*:*:delete").unwrap();
    assert_eq!(p.phase, PatternSegment::Wildcard);
    assert_eq!(p.item_type, PatternSegment::Wildcard);
    assert_eq!(p.operation, PatternSegment::Exact("delete".to_string()));
}

#[test]
fn test_parse_all_wildcards() {
    let p = ParsedPattern::parse("*:*:*").unwrap();
    assert_eq!(p.phase, PatternSegment::Wildcard);
    assert_eq!(p.item_type, PatternSegment::Wildcard);
    assert_eq!(p.operation, PatternSegment::Wildcard);
}

#[test]
fn test_parse_invalid_segment_count() {
    let err = ParsedPattern::parse("pre:issue").unwrap_err();
    assert!(matches!(err, HookError::InvalidPattern(_)));
}

#[test]
fn test_parse_invalid_phase() {
    let err = ParsedPattern::parse("during:issue:create").unwrap_err();
    assert!(matches!(err, HookError::InvalidPattern(_)));
}

#[test]
fn test_parse_custom_item_type() {
    // Custom item types should now be accepted
    let p = ParsedPattern::parse("pre:widget:create").unwrap();
    assert_eq!(p.item_type, PatternSegment::Exact("widget".to_string()));
}

#[test]
fn test_parse_invalid_operation() {
    let err = ParsedPattern::parse("pre:issue:explode").unwrap_err();
    assert!(matches!(err, HookError::InvalidPattern(_)));
}

#[test]
fn test_matches_exact() {
    let p = ParsedPattern::parse("pre:issue:create").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(!p.matches(Phase::Post, "issue", HookOperation::Create));
    assert!(!p.matches(Phase::Pre, "doc", HookOperation::Create));
    assert!(!p.matches(Phase::Pre, "issue", HookOperation::Delete));
}

#[test]
fn test_matches_wildcard_phase() {
    let p = ParsedPattern::parse("*:issue:create").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(p.matches(Phase::Post, "issue", HookOperation::Create));
    assert!(!p.matches(Phase::Pre, "doc", HookOperation::Create));
}

#[test]
fn test_matches_wildcard_item_type() {
    let p = ParsedPattern::parse("pre:*:create").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(p.matches(Phase::Pre, "doc", HookOperation::Create));
    assert!(!p.matches(Phase::Post, "issue", HookOperation::Create));
}

#[test]
fn test_matches_all_wildcards() {
    let p = ParsedPattern::parse("*:*:*").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(p.matches(Phase::Post, "doc", HookOperation::Delete));
}

