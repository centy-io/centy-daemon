use super::*;

#[test]
fn test_parse_github_https_with_git() {
    let result = parse_remote_url("https://github.com/my-org/my-repo.git");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "github.com".to_string(),
            org: "my-org".to_string(),
            repo: "my-repo".to_string(),
        })
    );
}

#[test]
fn test_parse_github_https_without_git() {
    let result = parse_remote_url("https://github.com/my-org/my-repo");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "github.com".to_string(),
            org: "my-org".to_string(),
            repo: "my-repo".to_string(),
        })
    );
}

#[test]
fn test_parse_github_ssh() {
    let result = parse_remote_url("git@github.com:my-org/my-repo.git");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "github.com".to_string(),
            org: "my-org".to_string(),
            repo: "my-repo".to_string(),
        })
    );
}

#[test]
fn test_parse_github_ssh_without_git() {
    let result = parse_remote_url("git@github.com:my-org/my-repo");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "github.com".to_string(),
            org: "my-org".to_string(),
            repo: "my-repo".to_string(),
        })
    );
}

#[test]
fn test_parse_gitlab_https() {
    let result = parse_remote_url("https://gitlab.com/org-name/repo");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "gitlab.com".to_string(),
            org: "org-name".to_string(),
            repo: "repo".to_string(),
        })
    );
}

#[test]
fn test_parse_gitlab_ssh() {
    let result = parse_remote_url("git@gitlab.com:org-name/repo.git");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "gitlab.com".to_string(),
            org: "org-name".to_string(),
            repo: "repo".to_string(),
        })
    );
}
