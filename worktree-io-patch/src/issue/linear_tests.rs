use super::*;

#[test]
fn test_parse_linear_shorthand() {
    let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
    let r = IssueRef::parse(&format!("acme/api@{uuid}")).unwrap();
    assert_eq!(
        r,
        IssueRef::Linear {
            owner: "acme".into(),
            repo: "api".into(),
            id: uuid.into(),
        }
    );
}

#[test]
fn test_parse_linear_shorthand_invalid_uuid() {
    let err = IssueRef::parse("acme/api@not-a-uuid").unwrap_err();
    assert!(err.to_string().contains("Invalid Linear issue UUID"));
}

#[test]
fn test_parse_linear_worktree_url() {
    let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
    let url = format!("worktree://open?owner=acme&repo=api&linear_id={uuid}");
    let r = IssueRef::parse(&url).unwrap();
    assert_eq!(
        r,
        IssueRef::Linear {
            owner: "acme".into(),
            repo: "api".into(),
            id: uuid.into(),
        }
    );
}

#[test]
fn test_parse_linear_worktree_url_with_editor() {
    let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
    let url = format!("worktree://open?owner=acme&repo=api&linear_id={uuid}&editor=cursor");
    let (r, opts) = IssueRef::parse_with_options(&url).unwrap();
    assert_eq!(
        r,
        IssueRef::Linear {
            owner: "acme".into(),
            repo: "api".into(),
            id: uuid.into(),
        }
    );
    assert_eq!(opts.editor.as_deref(), Some("cursor"));
}

#[test]
fn test_linear_workspace_dir_name() {
    let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
    let r = IssueRef::Linear {
        owner: "acme".into(),
        repo: "api".into(),
        id: uuid.into(),
    };
    assert_eq!(r.workspace_dir_name(), format!("linear-{uuid}"));
    assert_eq!(r.branch_name(), format!("linear-{uuid}"));
}

#[test]
fn test_linear_clone_url() {
    let r = IssueRef::Linear {
        owner: "acme".into(),
        repo: "api".into(),
        id: "9cad7a4b-9426-4788-9dbc-e784df999053".into(),
    };
    assert_eq!(r.clone_url(), "https://github.com/acme/api.git");
}

#[test]
fn test_linear_paths() {
    let uuid = "9cad7a4b-9426-4788-9dbc-e784df999053";
    let r = IssueRef::Linear {
        owner: "acme".into(),
        repo: "api".into(),
        id: uuid.into(),
    };
    assert!(r.bare_clone_path().ends_with("worktrees/github/acme/api"));
    assert!(r
        .temp_path()
        .ends_with(format!("worktrees/github/acme/api/linear-{uuid}")));
}
