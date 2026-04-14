use super::*;

#[test]
fn test_parse_jira_worktree_url() {
    let r = IssueRef::parse(
        "worktree://open?jira_host=acme.atlassian.net&jira_issue_key=PROJ-42&owner=acme&repo=api",
    )
    .unwrap();
    assert_eq!(
        r,
        IssueRef::Jira {
            host: "acme.atlassian.net".into(),
            issue_key: "PROJ-42".into(),
            owner: "acme".into(),
            repo: "api".into(),
        }
    );
}

#[test]
fn test_parse_jira_worktree_url_with_editor() {
    let (r, opts) = IssueRef::parse_with_options(
        "worktree://open?jira_host=acme.atlassian.net&jira_issue_key=PROJ-42&owner=acme&repo=api&editor=cursor",
    )
    .unwrap();
    assert_eq!(
        r,
        IssueRef::Jira {
            host: "acme.atlassian.net".into(),
            issue_key: "PROJ-42".into(),
            owner: "acme".into(),
            repo: "api".into(),
        }
    );
    assert_eq!(opts.editor.as_deref(), Some("cursor"));
}

#[test]
fn test_parse_jira_worktree_url_missing_host() {
    let err =
        IssueRef::parse("worktree://open?jira_issue_key=PROJ-42&owner=acme&repo=api").unwrap_err();
    assert!(err.to_string().contains("Missing 'jira_host'"));
}

#[test]
fn test_parse_jira_worktree_url_missing_owner() {
    let err = IssueRef::parse(
        "worktree://open?jira_host=acme.atlassian.net&jira_issue_key=PROJ-42&repo=api",
    )
    .unwrap_err();
    assert!(err.to_string().contains("Missing 'owner'"));
}

#[test]
fn test_parse_jira_browse_url_error() {
    let err = IssueRef::parse("https://acme.atlassian.net/browse/PROJ-42").unwrap_err();
    assert!(err
        .to_string()
        .contains("Jira browse URLs cannot be opened directly"));
}

#[test]
fn test_jira_workspace_dir_name() {
    let r = IssueRef::Jira {
        host: "acme.atlassian.net".into(),
        issue_key: "PROJ-42".into(),
        owner: "acme".into(),
        repo: "api".into(),
    };
    assert_eq!(r.workspace_dir_name(), "jira-proj-42");
    assert_eq!(r.branch_name(), "jira-proj-42");
}

#[test]
fn test_jira_clone_url() {
    let r = IssueRef::Jira {
        host: "acme.atlassian.net".into(),
        issue_key: "PROJ-42".into(),
        owner: "acme".into(),
        repo: "api".into(),
    };
    assert_eq!(r.clone_url(), "https://github.com/acme/api.git");
}

#[test]
fn test_jira_paths() {
    let r = IssueRef::Jira {
        host: "acme.atlassian.net".into(),
        issue_key: "PROJ-42".into(),
        owner: "acme".into(),
        repo: "api".into(),
    };
    assert!(r.bare_clone_path().ends_with("worktrees/github/acme/api"));
    assert!(r
        .temp_path()
        .ends_with("worktrees/github/acme/api/jira-proj-42"));
}
