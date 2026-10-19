use crate::server::proto::{CheckForUpdatesRequest, CheckForUpdatesResponse, GitHubRelease};
use crate::server::structured_error::StructuredError;
use crate::utils::CENTY_VERSION;
use serde::Deserialize;
use std::time::Duration;
use tonic::{Response, Status};

const DEFAULT_RELEASES_URL: &str = "https://api.github.com/repos/centy-io/centy-daemon/releases";

/// Env var that overrides the releases endpoint (used by tests).
const RELEASES_URL_ENV: &str = "CENTY_RELEASES_URL";

/// Env var that overrides the total request timeout in milliseconds (used by tests).
const TIMEOUT_MS_ENV: &str = "CENTY_RELEASES_TIMEOUT_MS";
const DEFAULT_TIMEOUT_MS: u64 = 10_000;
const CONNECT_TIMEOUT_MS: u64 = 5_000;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
    prerelease: bool,
    html_url: String,
}

async fn fetch_releases() -> Result<Vec<Release>, reqwest::Error> {
    let url = std::env::var(RELEASES_URL_ENV).unwrap_or_else(|_| DEFAULT_RELEASES_URL.to_string());
    let timeout_ms = std::env::var(TIMEOUT_MS_ENV)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_MS);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .connect_timeout(Duration::from_millis(CONNECT_TIMEOUT_MS.min(timeout_ms)))
        .build()?;
    let mut request = client
        .get(url)
        .header("User-Agent", "centy-daemon")
        .header("Accept", "application/vnd.github+json");
    // Optional token for higher GitHub rate limits.
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            request = request.bearer_auth(token);
        }
    }
    request
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<Release>>()
        .await
}

/// Newest non-prerelease version without the leading `v`; GitHub lists newest first.
fn latest_stable(releases: &[Release]) -> Option<String> {
    releases.iter().find(|r| !r.prerelease).map(|r| {
        r.tag_name
            .strip_prefix('v')
            .unwrap_or(&r.tag_name)
            .to_string()
    })
}

fn is_newer(current: &str, latest: &str) -> bool {
    match (
        semver::Version::parse(current),
        semver::Version::parse(latest),
    ) {
        (Ok(cur), Ok(lat)) => lat > cur,
        _ => false,
    }
}

pub async fn check_for_updates(
    _req: CheckForUpdatesRequest,
) -> Result<Response<CheckForUpdatesResponse>, Status> {
    let releases = match fetch_releases().await {
        Ok(r) => r,
        Err(e) => {
            let se = StructuredError::new(
                "",
                "GITHUB_API_ERROR",
                format!("Failed to fetch releases: {e}"),
            );
            return Ok(Response::new(CheckForUpdatesResponse {
                success: false,
                error: se.to_json(),
                current_version: CENTY_VERSION.to_string(),
                ..Default::default()
            }));
        }
    };
    let latest_version = latest_stable(&releases).unwrap_or_default();
    Ok(Response::new(CheckForUpdatesResponse {
        success: true,
        error: String::new(),
        current_version: CENTY_VERSION.to_string(),
        update_available: is_newer(CENTY_VERSION, &latest_version),
        latest_version,
        releases: releases
            .into_iter()
            .map(|r| GitHubRelease {
                tag_name: r.tag_name,
                name: r.name.unwrap_or_default(),
                published_at: r.published_at.unwrap_or_default(),
                prerelease: r.prerelease,
                html_url: r.html_url,
            })
            .collect(),
    }))
}
