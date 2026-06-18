use super::*;

#[test]
fn parse_https_remotes() {
    let (o, r) = parse_gitlab_remote_url("https://gitlab.com/acme/api.git").unwrap();
    assert_eq!((o.as_str(), r.as_str()), ("acme", "api"));
    let (o, r) = parse_gitlab_remote_url("https://gitlab.com/microsoft/vscode").unwrap();
    assert_eq!((o.as_str(), r.as_str()), ("microsoft", "vscode"));
}

#[test]
fn parse_ssh_remotes() {
    let (o, r) = parse_gitlab_remote_url("git@gitlab.com:acme/api.git").unwrap();
    assert_eq!((o.as_str(), r.as_str()), ("acme", "api"));
    let (o, r) = parse_gitlab_remote_url("git@gitlab.com:microsoft/vscode").unwrap();
    assert_eq!((o.as_str(), r.as_str()), ("microsoft", "vscode"));
}

#[test]
fn parse_non_gitlab_and_empty_owner_return_none() {
    assert!(parse_gitlab_remote_url("https://github.com/owner/repo.git").is_none());
    assert!(parse_gitlab_remote_url("git@github.com:owner/repo.git").is_none());
    assert!(parse_gitlab_remote_url("https://gitlab.com//repo").is_none());
}

#[test]
fn parse_gl_invalid_number() {
    let err = parse_gl("gl:abc").unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid issue number for gl shorthand"));
}

#[test]
fn parse_gitlab_url_valid() {
    let r = parse_gitlab_url("https://gitlab.com/acme/api/-/issues/42").unwrap();
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
fn parse_gitlab_url_invalid_path() {
    let err = parse_gitlab_url("https://gitlab.com/acme/api/issues/42").unwrap_err();
    assert!(err.to_string().contains("Expected GitLab issue URL"));
}

#[test]
fn parse_gitlab_url_invalid_number() {
    let err = parse_gitlab_url("https://gitlab.com/acme/api/-/issues/abc").unwrap_err();
    assert!(err.to_string().contains("Invalid issue number"));
}
