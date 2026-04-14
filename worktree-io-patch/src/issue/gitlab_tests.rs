use super::*;

#[test]
fn parse_gitlab_url() {
    let r = IssueRef::parse("https://gitlab.com/acme/api/-/issues/42").unwrap();
    assert_eq!(
        r,
        IssueRef::GitLab {
            owner: "acme".into(),
            repo: "api".into(),
            number: 42,
        }
    );
}

#[test]
fn parse_gitlab_worktree_url() {
    let r = IssueRef::parse("worktree://open?gitlab_host=gitlab.com&owner=acme&repo=api&issue=7")
        .unwrap();
    assert_eq!(
        r,
        IssueRef::GitLab {
            owner: "acme".into(),
            repo: "api".into(),
            number: 7,
        }
    );
}

#[test]
fn parse_gitlab_worktree_url_with_editor() {
    let (r, opts) = IssueRef::parse_with_options(
        "worktree://open?gitlab_host=gitlab.com&owner=acme&repo=api&issue=7&editor=cursor",
    )
    .unwrap();
    assert_eq!(
        r,
        IssueRef::GitLab {
            owner: "acme".into(),
            repo: "api".into(),
            number: 7,
        }
    );
    assert_eq!(opts.editor.as_deref(), Some("cursor"));
}

#[test]
fn workspace_dir_name() {
    let r = IssueRef::GitLab {
        owner: "acme".into(),
        repo: "api".into(),
        number: 42,
    };
    assert_eq!(r.workspace_dir_name(), "issue-42");
    assert_eq!(r.branch_name(), "issue-42");
}

#[test]
fn clone_url() {
    let r = IssueRef::GitLab {
        owner: "acme".into(),
        repo: "api".into(),
        number: 42,
    };
    assert_eq!(r.clone_url(), "https://gitlab.com/acme/api.git");
}

#[test]
fn paths() {
    let r = IssueRef::GitLab {
        owner: "acme".into(),
        repo: "api".into(),
        number: 42,
    };
    assert!(r.bare_clone_path().ends_with("worktrees/gitlab/acme/api"));
    assert!(r
        .temp_path()
        .ends_with("worktrees/gitlab/acme/api/issue-42"));
}
