use super::*;

#[test]
fn test_parse_bitbucket_https() {
    let result = parse_remote_url("https://bitbucket.org/my-team/my-project.git");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "bitbucket.org".to_string(),
            org: "my-team".to_string(),
            repo: "my-project".to_string(),
        })
    );
}

#[test]
fn test_parse_self_hosted() {
    let result = parse_remote_url("https://git.company.com/engineering/api-service.git");
    assert_eq!(
        result,
        Some(ParsedRemote {
            host: "git.company.com".to_string(),
            org: "engineering".to_string(),
            repo: "api-service".to_string(),
        })
    );
}
