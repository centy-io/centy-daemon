use anyhow::{Context, Result};

use crate::issue::IssueRef;

/// Parse a GitHub remote URL into `(owner, repo)`.
///
/// Supports HTTPS (`https://github.com/owner/repo[.git]`) and
/// SSH (`git@github.com:owner/repo[.git]`) formats.
/// Returns `None` if the URL is not a recognised GitHub remote.
pub(super) fn parse_github_remote_url(url: &str) -> Option<(String, String)> {
    let url = url.trim().trim_end_matches(".git");
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("git@github.com:"))?;
    let (owner, repo) = path.split_once('/')?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// Parse a `gh:<number>` shorthand into an [`IssueRef::GitHub`].
///
/// Reads the `origin` remote URL from the current git repository and resolves
/// the issue number against it.
///
/// # Errors
///
/// Returns an error if the number is invalid, the current directory cannot be
/// determined, the `origin` remote URL cannot be read, or the URL is not a
/// GitHub remote.
pub(super) fn parse_gh(s: &str) -> Result<IssueRef> {
    let num_str = s
        .strip_prefix("gh:")
        .expect("caller checked starts_with(\"gh:\")");
    let Ok(number) = num_str.parse::<u64>() else {
        return Err(anyhow::anyhow!(
            "Invalid issue number for gh shorthand: {num_str:?} — expected a positive integer"
        ));
    };
    // LLVM_COV_EXCL_START
    let cwd = std::env::current_dir().context("Could not determine current directory")?;
    let remote_url = crate::git::get_remote_url(&cwd, "origin").context(
        "Could not get GitHub remote URL — is this a git repository with an 'origin' remote?",
    )?;
    let (owner, repo) = parse_github_remote_url(&remote_url)
        .ok_or_else(|| anyhow::anyhow!("Remote URL {remote_url:?} is not a GitHub URL"))?;
    Ok(IssueRef::GitHub {
        owner,
        repo,
        number,
    })
    // LLVM_COV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_https_remotes() {
        let (o, r) = parse_github_remote_url("https://github.com/acme/api.git").unwrap();
        assert_eq!((o.as_str(), r.as_str()), ("acme", "api"));
        let (o, r) = parse_github_remote_url("https://github.com/microsoft/vscode").unwrap();
        assert_eq!((o.as_str(), r.as_str()), ("microsoft", "vscode"));
    }

    #[test]
    fn parse_ssh_remotes() {
        let (o, r) = parse_github_remote_url("git@github.com:acme/api.git").unwrap();
        assert_eq!((o.as_str(), r.as_str()), ("acme", "api"));
        let (o, r) = parse_github_remote_url("git@github.com:microsoft/vscode").unwrap();
        assert_eq!((o.as_str(), r.as_str()), ("microsoft", "vscode"));
    }

    #[test]
    fn parse_non_github_and_empty_owner_return_none() {
        assert!(parse_github_remote_url("https://gitlab.com/owner/repo.git").is_none());
        assert!(parse_github_remote_url("git@gitlab.com:owner/repo.git").is_none());
        assert!(parse_github_remote_url("https://github.com//repo").is_none());
    }

    #[test]
    fn parse_gh_invalid_number() {
        let err = parse_gh("gh:abc").unwrap_err();
        assert!(err
            .to_string()
            .contains("Invalid issue number for gh shorthand"));
    }
}
