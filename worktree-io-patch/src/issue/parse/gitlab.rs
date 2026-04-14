use anyhow::{bail, Context, Result};
use url::Url;

use crate::issue::IssueRef;

/// Parse a GitLab remote URL into `(owner, repo)`.
///
/// Supports HTTPS (`https://gitlab.com/owner/repo[.git]`) and
/// SSH (`git@gitlab.com:owner/repo[.git]`) formats.
/// Returns `None` if the URL is not a recognised GitLab remote.
pub(super) fn parse_gitlab_remote_url(url: &str) -> Option<(String, String)> {
    let url = url.trim().trim_end_matches(".git");
    let path = url
        .strip_prefix("https://gitlab.com/")
        .or_else(|| url.strip_prefix("git@gitlab.com:"))?;
    let (owner, repo) = path.split_once('/')?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// Parse a `https://gitlab.com/<owner>/<repo>/-/issues/<N>` URL into an
/// [`IssueRef::GitLab`].
///
/// # Errors
///
/// Returns an error if the URL does not match the expected GitLab issue URL
/// pattern or if the issue number is invalid.
pub(super) fn parse_gitlab_url(s: &str) -> Result<IssueRef> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;

    let segments: Vec<&str> = url
        .path_segments()
        .context("URL has no path")?
        .filter(|s| !s.is_empty())
        .collect();

    // Expected: /<owner>/<repo>/-/issues/<N>
    if segments.len() < 5 || segments[2] != "-" || segments[3] != "issues" {
        bail!(
            "Expected GitLab issue URL like \
             https://gitlab.com/owner/repo/-/issues/42, got: {s}"
        );
    }

    let owner = segments[0].to_string();
    let repo = segments[1].to_string();
    let number = segments[4]
        .parse::<u64>()
        .with_context(|| format!("Invalid issue number in URL: {}", segments[4]))?;

    Ok(IssueRef::GitLab {
        owner,
        repo,
        number,
    })
}

/// Parse a `gl:<number>` shorthand into an [`IssueRef::GitLab`].
///
/// Reads the `origin` remote URL from the current git repository and resolves
/// the issue number against it.
///
/// # Errors
///
/// Returns an error if the number is invalid, the current directory cannot be
/// determined, the `origin` remote URL cannot be read, or the URL is not a
/// GitLab remote.
pub(super) fn parse_gl(s: &str) -> Result<IssueRef> {
    let num_str = s
        .strip_prefix("gl:")
        .expect("caller checked starts_with(\"gl:\")");
    let Ok(number) = num_str.parse::<u64>() else {
        return Err(anyhow::anyhow!(
            "Invalid issue number for gl shorthand: {num_str:?} — expected a positive integer"
        ));
    };
    // LLVM_COV_EXCL_START
    let cwd = std::env::current_dir().context("Could not determine current directory")?;
    let remote_url = crate::git::get_remote_url(&cwd, "origin").context(
        "Could not get GitLab remote URL — is this a git repository with an 'origin' remote?",
    )?;
    let (owner, repo) = parse_gitlab_remote_url(&remote_url)
        .ok_or_else(|| anyhow::anyhow!("Remote URL {remote_url:?} is not a GitLab URL"))?;
    Ok(IssueRef::GitLab {
        owner,
        repo,
        number,
    })
    // LLVM_COV_EXCL_STOP
}

#[cfg(test)]
#[path = "gitlab_parse_tests.rs"]
mod tests;
