// cspell:ignore Fgithub Facme Fapi Fissues
use super::*;

#[test]
fn test_parse_unrecognized_input() {
    let err = IssueRef::parse("not-a-valid-ref").unwrap_err();
    assert!(err.to_string().contains("Could not parse issue reference"));
}

#[test]
fn test_parse_github_url_wrong_path() {
    let err = IssueRef::parse("https://github.com/owner/repo/pulls/1").unwrap_err();
    assert!(err.to_string().contains("Expected GitHub issue URL"));
}

#[test]
fn test_parse_worktree_url_param() {
    let r =
        IssueRef::parse("worktree://open?url=https%3A%2F%2Fgithub.com%2Facme%2Fapi%2Fissues%2F5")
            .unwrap();
    assert_eq!(
        r,
        IssueRef::GitHub {
            owner: "acme".into(),
            repo: "api".into(),
            number: 5
        }
    );
}

#[test]
fn test_parse_worktree_url_unknown_param() {
    let r = IssueRef::parse("worktree://open?owner=a&repo=b&issue=1&foo=bar").unwrap();
    assert_eq!(
        r,
        IssueRef::GitHub {
            owner: "a".into(),
            repo: "b".into(),
            number: 1
        }
    );
}

#[test]
fn test_parse_worktree_url_invalid_linear_uuid() {
    let err = IssueRef::parse("worktree://open?owner=a&repo=b&linear_id=not-a-uuid").unwrap_err();
    assert!(err.to_string().contains("Invalid Linear issue UUID"));
}

#[test]
fn test_shorthand_empty_owner_at() {
    let err = IssueRef::parse("/repo@550e8400-e29b-41d4-a716-446655440000").unwrap_err();
    assert!(err.to_string().contains("Invalid shorthand format"));
}

#[test]
fn test_shorthand_empty_repo_hash() {
    let err = IssueRef::parse("owner/#42").unwrap_err();
    assert!(err.to_string().contains("Invalid shorthand format"));
}

#[test]
fn test_shorthand_non_numeric_issue() {
    let err = IssueRef::parse("owner/repo#abc").unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid issue number in shorthand"));
}

#[test]
fn test_parse_github_url_non_numeric_issue() {
    let err = IssueRef::parse("https://github.com/owner/repo/issues/abc").unwrap_err();
    assert!(err.to_string().contains("Invalid issue number in URL"));
}

#[test]
fn test_parse_worktree_url_invalid_issue_number() {
    let err = IssueRef::parse("worktree://open?owner=a&repo=b&issue=abc").unwrap_err();
    assert!(err.to_string().contains("Invalid issue number"));
}
