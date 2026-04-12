use super::*;

#[test]
fn test_parse_valid_pattern() {
    let p = ParsedPattern::parse("issue.creating").unwrap();
    assert_eq!(p.item_type, PatternSegment::Exact("issue".to_string()));
    assert_eq!(p.event, PatternSegment::Exact("creating".to_string()));
}

#[test]
fn test_parse_wildcard_pattern() {
    let p = ParsedPattern::parse("*.deleted").unwrap();
    assert_eq!(p.item_type, PatternSegment::Wildcard);
    assert_eq!(p.event, PatternSegment::Exact("deleted".to_string()));
}

#[test]
fn test_parse_all_wildcards() {
    let p = ParsedPattern::parse("*.*").unwrap();
    assert_eq!(p.item_type, PatternSegment::Wildcard);
    assert_eq!(p.event, PatternSegment::Wildcard);
}

#[test]
fn test_parse_invalid_segment_count() {
    let err = ParsedPattern::parse("issue").unwrap_err();
    assert!(matches!(err, HookError::InvalidPattern(_)));
}

#[test]
fn test_parse_invalid_event() {
    let err = ParsedPattern::parse("issue.explode").unwrap_err();
    assert!(matches!(err, HookError::InvalidPattern(_)));
}

#[test]
fn test_parse_custom_item_type() {
    // Custom item types should be accepted
    let p = ParsedPattern::parse("widget.creating").unwrap();
    assert_eq!(p.item_type, PatternSegment::Exact("widget".to_string()));
}

#[test]
fn test_matches_exact() {
    let p = ParsedPattern::parse("issue.creating").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(!p.matches(Phase::Post, "issue", HookOperation::Create));
    assert!(!p.matches(Phase::Pre, "doc", HookOperation::Create));
    assert!(!p.matches(Phase::Pre, "issue", HookOperation::Delete));
}

#[test]
fn test_matches_wildcard_item_type() {
    let p = ParsedPattern::parse("*.creating").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(p.matches(Phase::Pre, "doc", HookOperation::Create));
    assert!(!p.matches(Phase::Post, "issue", HookOperation::Create));
}

#[test]
fn test_matches_wildcard_event() {
    let p = ParsedPattern::parse("issue.*").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(p.matches(Phase::Post, "issue", HookOperation::Create));
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Delete));
    assert!(!p.matches(Phase::Pre, "doc", HookOperation::Create));
}

#[test]
fn test_matches_all_wildcards() {
    let p = ParsedPattern::parse("*.*").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(p.matches(Phase::Post, "doc", HookOperation::Delete));
}
