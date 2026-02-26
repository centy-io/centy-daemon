//! Git remote URL parsing utilities for organization inference.
//!
//! Supports various URL formats including:
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
/// Supports HTTPS, HTTP, and SSH formats, including self-hosted.
/// Returns `None` if the URL format is not recognized.
pub fn parse_remote_url(url: &str) -> Option<ParsedRemote> {
    let url = url.trim();
    if let Some(ssh_part) = url.strip_prefix("git@") {
        return parse_ssh_url(ssh_part);
    }
    if url.starts_with("https://") || url.starts_with("http://") {
        return parse_https_url(url);
    }
    None
}

fn parse_ssh_url(ssh_part: &str) -> Option<ParsedRemote> {
    let (host, path) = ssh_part.split_once(':')?;
    parse_path_segments(host, path)
}

fn parse_https_url(url: &str) -> Option<ParsedRemote> {
    let url = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let (host, path) = url.split_once('/')?;
    parse_path_segments(host, path)
}

fn parse_path_segments(host: &str, path: &str) -> Option<ParsedRemote> {
    let path = path.strip_suffix(".git").unwrap_or(path);
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let org = parts.first()?;
    let repo = parts.get(1)?;
    Some(ParsedRemote { host: host.to_string(), org: (*org).to_string(), repo: (*repo).to_string() })
}

#[cfg(test)]
#[path = "remote_tests_1.rs"]
mod tests_1;
#[cfg(test)]
#[path = "remote_tests_2.rs"]
mod tests_2;
#[cfg(test)]
#[path = "remote_tests_3.rs"]
mod tests_3;
