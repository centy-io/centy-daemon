use serde::Deserialize;
use thiserror::Error;

const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/centy-io/centy-daemon/releases";

#[derive(Error, Debug)]
pub enum GitHubReleasesError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub published_at: Option<String>,
    pub prerelease: bool,
    pub html_url: String,
}

/// Fetch all releases from the GitHub API.
///
/// When the `GITHUB_TOKEN` environment variable is set, it is sent as a
/// bearer token to benefit from higher rate limits.
pub async fn fetch_releases() -> Result<Vec<GitHubRelease>, GitHubReleasesError> {
    let client = reqwest::Client::new();
    let mut request = client
        .get(GITHUB_RELEASES_URL)
        .header("User-Agent", "centy-daemon")
        .header("Accept", "application/vnd.github+json");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            request = request.bearer_auth(token);
        }
    }

    let releases: Vec<GitHubRelease> = request.send().await?.json().await?;
    Ok(releases)
}

/// Return the latest non-prerelease version string (without the leading `v`).
///
/// Returns `None` when the release list is empty or contains only
/// pre-releases.
#[must_use]
pub fn latest_stable_version(releases: &[GitHubRelease]) -> Option<String> {
    releases
        .iter()
        .filter(|r| !r.prerelease)
        .map(|r| r.tag_name.strip_prefix('v').unwrap_or(&r.tag_name))
        .next()
        .map(String::from)
}

/// Compare two semver strings and return `true` when `latest` is newer than
/// `current`.
#[must_use]
pub fn is_update_available(current: &str, latest: &str) -> bool {
    match (
        semver::Version::parse(current),
        semver::Version::parse(latest),
    ) {
        (Ok(cur), Ok(lat)) => lat > cur,
        _ => false,
    }
}

#[cfg(test)]
#[path = "github_releases_tests.rs"]
mod tests;
