use anyhow::{bail, Context, Result};

use crate::issue::IssueRef;

/// Jira browse URLs (`https://{host}/browse/{ISSUE-KEY}`) do not contain the
/// GitHub repository needed to clone the code.  Return a helpful error so
/// users know to use the full `worktree://` deep-link format instead.
pub(super) fn parse_jira_browse_url(s: &str) -> Result<IssueRef> {
    bail!(
        "Jira browse URLs cannot be opened directly — the GitHub repository is not part of the URL.\n\
         Use the worktree:// deep-link format instead:\n\
         worktree://open?jira_host=<host>&jira_issue_key=<PROJ-42>&owner=<owner>&repo=<repo>\n\
         Got: {s}"
    )
}

/// Build an [`IssueRef::Jira`] from raw `worktree://` query params.
pub(super) fn resolve_worktree_params(
    host: Option<String>,
    issue_key: String,
    owner: Option<String>,
    repo: Option<String>,
) -> Result<IssueRef> {
    Ok(IssueRef::Jira {
        host: host.context("Missing 'jira_host' query param")?,
        issue_key,
        owner: owner.context("Missing 'owner' query param")?,
        repo: repo.context("Missing 'repo' query param")?,
    })
}
