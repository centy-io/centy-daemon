use super::*;
#[test]
fn test_parse_shorthand() {
    let r = IssueRef::parse("owner/repo#42").unwrap();
    assert_eq!(
        r,
        IssueRef::GitHub {
            owner: "owner".into(),
            repo: "repo".into(),
            number: 42
        }
    );
}
#[test]
fn test_parse_github_url() {
    let r = IssueRef::parse("https://github.com/microsoft/vscode/issues/12345").unwrap();
    assert_eq!(
        r,
        IssueRef::GitHub {
            owner: "microsoft".into(),
            repo: "vscode".into(),
            number: 12345
        }
    );
}
#[test]
fn test_parse_worktree_url() {
    let r = IssueRef::parse("worktree://open?owner=acme&repo=api&issue=7").unwrap();
    assert_eq!(
        r,
        IssueRef::GitHub {
            owner: "acme".into(),
            repo: "api".into(),
            number: 7
        }
    );
}
#[test]
fn test_parse_worktree_url_with_editor_symbolic() {
    let (r, opts) =
        IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&issue=42&editor=cursor")
            .unwrap();
    assert_eq!(
        r,
        IssueRef::GitHub {
            owner: "acme".into(),
            repo: "api".into(),
            number: 42
        }
    );
    assert_eq!(opts.editor.as_deref(), Some("cursor"));
}
#[test]
fn test_parse_worktree_url_with_editor_raw_command() {
    let (r, opts) = IssueRef::parse_with_options(
        "worktree://open?owner=acme&repo=api&issue=42&editor=my-editor%20.",
    )
    .unwrap();
    assert_eq!(
        r,
        IssueRef::GitHub {
            owner: "acme".into(),
            repo: "api".into(),
            number: 42
        }
    );
    assert_eq!(opts.editor.as_deref(), Some("my-editor ."));
}
#[test]
fn test_parse_with_options_no_editor() {
    let (_r, opts) =
        IssueRef::parse_with_options("worktree://open?owner=acme&repo=api&issue=42").unwrap();
    assert!(opts.editor.is_none());
}
#[test]
fn test_parse_with_options_non_deep_link() {
    let (_r, opts) = IssueRef::parse_with_options("acme/api#42").unwrap();
    assert!(opts.editor.is_none());
}
#[test]
fn test_paths() {
    let r = IssueRef::GitHub {
        owner: "acme".into(),
        repo: "api".into(),
        number: 7,
    };
    assert!(r.bare_clone_path().ends_with("worktrees/github/acme/api"));
    assert!(r.temp_path().ends_with("worktrees/github/acme/api/issue-7"));
}
#[test]
fn test_clone_url() {
    let r = IssueRef::GitHub {
        owner: "acme".into(),
        repo: "api".into(),
        number: 7,
    };
    assert_eq!(r.clone_url(), "https://github.com/acme/api.git");
}
