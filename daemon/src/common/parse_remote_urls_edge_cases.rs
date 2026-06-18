use super::*;

#[test]
fn test_parse_http_url() {
    let result = parse_remote_url("http://github.com/my-org/my-repo.git");
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
fn test_parse_invalid_url() {
    assert_eq!(parse_remote_url("not-a-url"), None);
}

#[test]
fn test_parse_empty_url() {
    assert_eq!(parse_remote_url(""), None);
}

#[test]
fn test_parse_url_with_whitespace() {
    let result = parse_remote_url("  https://github.com/my-org/my-repo.git  ");
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
fn test_parse_url_missing_repo() {
    // URL with only org, no repo
    assert_eq!(parse_remote_url("https://github.com/my-org"), None);
}

#[test]
fn test_parse_url_with_nested_paths() {
    // URLs with more than 2 path segments should still work (take first 2)
    let result = parse_remote_url("https://gitlab.com/group/subgroup/repo.git");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "gitlab.com".to_string(),
            org: "group".to_string(),
            repo: "subgroup".to_string(),
        })
    );
}
