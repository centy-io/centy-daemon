use anyhow::{bail, Context, Result};
use url::Url;

use crate::issue::IssueRef;

pub(super) fn parse_github_url(s: &str) -> Result<IssueRef> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;

    let segments: Vec<&str> = url
        .path_segments()
        .context("URL has no path")?
        .filter(|s| !s.is_empty())
        .collect();

    if segments.len() < 4 || segments[2] != "issues" {
        bail!("Expected GitHub issue URL like https://github.com/owner/repo/issues/42, got: {s}");
    }

    let owner = segments[0].to_string();
    let repo = segments[1].to_string();
    let number = segments[3]
        .parse::<u64>()
        .with_context(|| format!("Invalid issue number in URL: {}", segments[3]))?;

    Ok(IssueRef::GitHub {
        owner,
        repo,
        number,
    })
}
