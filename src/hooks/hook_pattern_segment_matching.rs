use super::*;

#[test]
fn test_pattern_segment_eq() {
    assert_eq!(PatternSegment::Wildcard, PatternSegment::Wildcard);
    assert_eq!(
        PatternSegment::Exact("issue".to_string()),
        PatternSegment::Exact("issue".to_string())
    );
    assert_ne!(
        PatternSegment::Wildcard,
        PatternSegment::Exact("*".to_string())
    );
}

#[test]
fn test_all_pre_operations() {
    let ops = [
        ("creating", HookOperation::Create),
        ("updating", HookOperation::Update),
        ("deleting", HookOperation::Delete),
        ("soft-deleting", HookOperation::SoftDelete),
        ("restoring", HookOperation::Restore),
        ("moving", HookOperation::Move),
        ("duplicating", HookOperation::Duplicate),
    ];

    for (event, op) in &ops {
        let pattern = format!("issue.{event}");
        let p = ParsedPattern::parse(&pattern)
            .unwrap_or_else(|_| panic!("Should parse pattern: {pattern}"));
        assert!(
            p.matches(Phase::Pre, "issue", *op),
            "Pattern '{pattern}' should match pre-{op:?}"
        );
        assert!(
            !p.matches(Phase::Post, "issue", *op),
            "Pattern '{pattern}' should not match post-{op:?}"
        );
    }
}

#[test]
fn test_all_post_operations() {
    let ops = [
        ("created", HookOperation::Create),
        ("updated", HookOperation::Update),
        ("deleted", HookOperation::Delete),
        ("soft-deleted", HookOperation::SoftDelete),
        ("restored", HookOperation::Restore),
        ("moved", HookOperation::Move),
        ("duplicated", HookOperation::Duplicate),
    ];

    for (event, op) in &ops {
        let pattern = format!("issue.{event}");
        let p = ParsedPattern::parse(&pattern)
            .unwrap_or_else(|_| panic!("Should parse pattern: {pattern}"));
        assert!(
            p.matches(Phase::Post, "issue", *op),
            "Pattern '{pattern}' should match post-{op:?}"
        );
        assert!(
            !p.matches(Phase::Pre, "issue", *op),
            "Pattern '{pattern}' should not match pre-{op:?}"
        );
    }
}

#[test]
fn test_specificity() {
    assert_eq!(ParsedPattern::parse("*.*").unwrap().specificity(), 0);
    assert_eq!(ParsedPattern::parse("issue.*").unwrap().specificity(), 1);
    assert_eq!(ParsedPattern::parse("*.creating").unwrap().specificity(), 1);
    assert_eq!(
        ParsedPattern::parse("issue.creating").unwrap().specificity(),
        2
    );
}

#[test]
fn test_soft_delete_pattern() {
    let p = ParsedPattern::parse("issue.soft-deleted").unwrap();
    assert!(p.matches(Phase::Post, "issue", HookOperation::SoftDelete));
    assert!(!p.matches(Phase::Post, "issue", HookOperation::Delete));
}

#[test]
fn test_all_item_types() {
    let p = ParsedPattern::parse("asset.creating").unwrap();
    assert!(p.matches(Phase::Pre, "asset", HookOperation::Create));

    let p = ParsedPattern::parse("link.creating").unwrap();
    assert!(p.matches(Phase::Pre, "link", HookOperation::Create));

    let p = ParsedPattern::parse("user.creating").unwrap();
    assert!(p.matches(Phase::Pre, "user", HookOperation::Create));

    let p = ParsedPattern::parse("task.creating").unwrap();
    assert!(p.matches(Phase::Pre, "task", HookOperation::Create));
}

#[test]
fn test_empty_item_type_segment_is_error() {
    let err = ParsedPattern::parse(".creating").unwrap_err();
    assert!(
        matches!(err, HookError::InvalidPattern(_)),
        "Expected InvalidPattern for empty item type segment"
    );
}

#[test]
fn test_wildcard_item_type_matches_any() {
    let p = ParsedPattern::parse("*.creating").unwrap();
    assert!(p.matches(Phase::Pre, "issue", HookOperation::Create));
    assert!(p.matches(Phase::Pre, "custom-type", HookOperation::Create));
    assert!(!p.matches(Phase::Post, "issue", HookOperation::Create));
}

#[test]
fn test_segment_matches_exact_false_when_different() {
    let p = ParsedPattern::parse("issue.creating").unwrap();
    assert!(!p.matches(Phase::Pre, "issue", HookOperation::Update));
    assert!(!p.matches(Phase::Pre, "bug", HookOperation::Create));
}
