//! Git remote URL parsing utilities for organization inference.
//!
//! Uses the `git-url-parse` crate for robust URL parsing across formats:
//! - HTTPS: `https://github.com/org-name/repo.git`
//! - SSH: `git@github.com:org-name/repo.git`
//! - Self-hosted: `https://git.company.com/org-name/repo.git`

use git_url_parse::GitUrl;

/// Result of parsing a git remote URL
#[derive(Debug, Clone, PartialEq, Eq)]
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
/// Supports HTTPS, HTTP, and SSH formats, including self-hosted.
/// Returns `None` if the URL format is not recognized.
#[must_use]
pub fn parse_remote_url(url: &str) -> Option<ParsedRemote> {
    let parsed = GitUrl::parse(url.trim()).ok()?;
    let host = parsed.host()?.to_string();
    let path = parsed.path();
    extract_org_and_repo(&host, path)
}

/// Extract the first two path segments as org and repo from a parsed path.
fn extract_org_and_repo(host: &str, path: &str) -> Option<ParsedRemote> {
    let clean = path
        .strip_suffix(".git")
        .unwrap_or(path)
        .trim_start_matches('/');
    let mut parts = clean.split('/').filter(|s| !s.is_empty());
    let org = parts.next()?;
    let repo = parts.next()?;
    Some(ParsedRemote {
        host: host.to_string(),
        org: org.to_string(),
        repo: repo.to_string(),
    })
}

#[cfg(test)]
#[path = "parse_bitbucket_and_self_hosted.rs"]
mod parse_bitbucket_and_self_hosted;
#[cfg(test)]
#[path = "parse_github_urls.rs"]
mod parse_github_urls;
#[cfg(test)]
#[path = "parse_remote_urls_edge_cases.rs"]
mod parse_remote_urls_edge_cases;
