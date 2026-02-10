//! Git remote URL parsing utilities for organization inference.
//!
//! This module provides utilities for parsing git remote URLs to extract
//! organization information. Supports various URL formats including:
//! - HTTPS: `https://github.com/org-name/repo.git`
//! - SSH: `git@github.com:org-name/repo.git`
//! - Self-hosted: `https://git.company.com/org-name/repo.git`

/// Result of parsing a git remote URL
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRemote {
    /// The hosting platform (e.g., "github.com", "gitlab.com", "bitbucket.org")
    pub host: String,
    /// The organization/user name (first path segment)
    pub org: String,
    /// The repository name (second path segment, without .git)
    pub repo: String,
}

/// Parse a git remote URL to extract organization information.
///
/// Supports:
/// - HTTPS: `https://github.com/org-name/repo.git`
/// - HTTP: `http://github.com/org-name/repo.git`
/// - SSH: `git@github.com:org-name/repo.git`
/// - Self-hosted: `https://git.company.com/org-name/repo.git`
///
/// # Arguments
/// * `url` - The git remote URL to parse
///
/// # Returns
/// * `Some(ParsedRemote)` if the URL was successfully parsed
/// * `None` if the URL format is not recognized
pub fn parse_remote_url(url: &str) -> Option<ParsedRemote> {
    let url = url.trim();

    // Handle SSH format: git@host:org/repo.git
    if let Some(ssh_part) = url.strip_prefix("git@") {
        return parse_ssh_url(ssh_part);
    }

    // Handle HTTPS/HTTP format
    if url.starts_with("https://") || url.starts_with("http://") {
        return parse_https_url(url);
    }

    None
}

fn parse_ssh_url(ssh_part: &str) -> Option<ParsedRemote> {
    // Format: host:org/repo.git
    let (host, path) = ssh_part.split_once(':')?;
    parse_path_segments(host, path)
}

fn parse_https_url(url: &str) -> Option<ParsedRemote> {
    let url = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let (host, path) = url.split_once('/')?;
    parse_path_segments(host, path)
}

fn parse_path_segments(host: &str, path: &str) -> Option<ParsedRemote> {
    let path = path.strip_suffix(".git").unwrap_or(path);
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    let org = parts.first()?;
    let repo = parts.get(1)?;

    Some(ParsedRemote {
        host: host.to_string(),
        org: (*org).to_string(),
        repo: (*repo).to_string(),
    })
}

#[cfg(test)]
mod tests {
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
}
