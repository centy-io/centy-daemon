use super::*;

#[test]
fn test_pattern_segment_eq() {
    assert_eq!(PatternSegment::Wildcard, PatternSegment::Wildcard);
    assert_eq!(
        PatternSegment::Exact("pre".to_string()),
        PatternSegment::Exact("pre".to_string())
    );
    assert_ne!(
        PatternSegment::Wildcard,
        PatternSegment::Exact("*".to_string())
    );
}

#[test]
fn test_all_operations() {
    let ops = [
        ("create", HookOperation::Create),
        ("update", HookOperation::Update),
        ("delete", HookOperation::Delete),
        ("soft-delete", HookOperation::SoftDelete),
        ("restore", HookOperation::Restore),
        ("move", HookOperation::Move),
        ("duplicate", HookOperation::Duplicate),
    ];

    for (name, op) in &ops {
        let pattern = format!("pre:issue:{name}");
        let p = ParsedPattern::parse(&pattern)
            .unwrap_or_else(|_| panic!("Should parse pattern: {pattern}"));
        assert!(
            p.matches(Phase::Pre, "issue", *op),
            "Pattern '{pattern}' should match"
        );
    }
}

#[test]
fn test_specificity() {
    assert_eq!(ParsedPattern::parse("*:*:*").unwrap().specificity(), 0);
    assert_eq!(ParsedPattern::parse("pre:*:*").unwrap().specificity(), 1);
    assert_eq!(
        ParsedPattern::parse("pre:issue:*").unwrap().specificity(),
        2
    );
    assert_eq!(
        ParsedPattern::parse("pre:issue:create")
            .unwrap()
            .specificity(),
        3
    );
    assert_eq!(ParsedPattern::parse("*:*:delete").unwrap().specificity(), 1);
    assert_eq!(
        ParsedPattern::parse("*:issue:delete")
            .unwrap()
            .specificity(),
        2
    );
}

#[test]
fn test_soft_delete_pattern() {
    let p = ParsedPattern::parse("post:issue:soft-delete").unwrap();
    assert!(p.matches(Phase::Post, "issue", HookOperation::SoftDelete));
    assert!(!p.matches(Phase::Post, "issue", HookOperation::Delete));
}

#[test]
fn test_all_item_types() {
    let p = ParsedPattern::parse("pre:asset:create").unwrap();
    assert!(p.matches(Phase::Pre, "asset", HookOperation::Create));

    let p = ParsedPattern::parse("pre:link:create").unwrap();
    assert!(p.matches(Phase::Pre, "link", HookOperation::Create));

    let p = ParsedPattern::parse("pre:user:create").unwrap();
    assert!(p.matches(Phase::Pre, "user", HookOperation::Create));

    let p = ParsedPattern::parse("pre:task:create").unwrap();
    assert!(p.matches(Phase::Pre, "task", HookOperation::Create));
}
