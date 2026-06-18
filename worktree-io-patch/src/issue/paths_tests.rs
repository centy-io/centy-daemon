use super::super::IssueRef;

#[test]
fn test_bare_clone_path_rooted_false() {
    let r = IssueRef::GitHub {
        owner: "acme".into(),
        repo: "api".into(),
        number: 7,
    };
    assert!(r
        .bare_clone_path_rooted(false)
        .ends_with("worktrees/github/acme/api"));
}

#[test]
fn test_bare_clone_path_rooted_true() {
    let r = IssueRef::GitHub {
        owner: "acme".into(),
        repo: "api".into(),
        number: 7,
    };
    let path = r.bare_clone_path_rooted(true);
    assert!(path.starts_with(std::env::temp_dir()));
    assert!(path.ends_with("worktrees/github/acme/api"));
}

#[test]
fn test_temp_path_rooted_true() {
    let r = IssueRef::GitHub {
        owner: "acme".into(),
        repo: "api".into(),
        number: 7,
    };
    let path = r.bare_clone_path_rooted(true).join(r.workspace_dir_name());
    assert!(path.starts_with(std::env::temp_dir()));
    assert!(path.ends_with("worktrees/github/acme/api/issue-7"));
}
