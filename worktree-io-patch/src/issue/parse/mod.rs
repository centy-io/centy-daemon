mod azure;
mod centy;
mod detect;
mod gh;
mod github;
mod gitlab;
mod jira;
mod options;
mod shorthand;
mod worktree_url;

use anyhow::{bail, Result};

use super::IssueRef;

impl IssueRef {
    /// Parse any of the supported input formats:
    /// - `https://github.com/owner/repo/issues/42`
    /// - `worktree://open?owner=X&repo=Y&issue=42`
    /// - `worktree://open?url=<encoded-github-url>`
    /// - `worktree://open?owner=X&repo=Y&linear_id=<uuid>`
    /// - `owner/repo#42`
    /// - `owner/repo@<linear-uuid>`
    /// - `centy:<number>` (context-aware: finds nearest `.centy/` ancestor)
    /// - `gh:<number>` (context-aware: resolves against the `origin` GitHub remote)
    /// - `owner/repo` (ad-hoc: auto-generates a random branch name)
    ///
    /// # Errors
    ///
    /// Returns an error if `s` does not match any supported format or if the
    /// extracted values (e.g. issue number) are invalid.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        if s.starts_with("worktree://") {
            let (issue, _opts) = worktree_url::parse_worktree_url(s)?;
            return Ok(issue);
        }

        if s.starts_with("https://github.com") || s.starts_with("http://github.com") {
            return github::parse_github_url(s);
        }
        if s.starts_with("https://dev.azure.com") || s.starts_with("http://dev.azure.com") {
            return azure::parse_azure_devops_url(s);
        }
        if s.starts_with("https://gitlab.com") || s.starts_with("http://gitlab.com") {
            return gitlab::parse_gitlab_url(s);
        }

        if s.contains(".atlassian.net/browse/") {
            return jira::parse_jira_browse_url(s);
        }

        if s.starts_with("centy:") {
            return centy::parse_centy(s);
        }

        if s.starts_with("gh:") {
            return gh::parse_gh(s);
        }

        if s.starts_with("gl:") {
            return gitlab::parse_gl(s);
        }

        if let Some(result) = shorthand::try_parse_shorthand(s) {
            return result;
        }

        if let Some((owner, repo)) = s.split_once('/') {
            if !owner.is_empty() && !repo.is_empty() && !repo.contains('/') {
                return Ok(Self::Adhoc {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    name: crate::name_gen::generate_name(),
                });
            }
        }

        bail!(
            "Could not parse issue reference: {s:?}\n\
             Supported formats:\n\
             - https://github.com/owner/repo/issues/42\n\
             - https://gitlab.com/owner/repo/-/issues/42\n\
             - https://dev.azure.com/org/project/_workitems/edit/42\n\
             - worktree://open?owner=owner&repo=repo&issue=42\n\
             - worktree://open?owner=owner&repo=repo&linear_id=<uuid>\n\
             - worktree://open?org=org&project=project&repo=repo&work_item_id=42\n\
             - worktree://open?jira_host=host&jira_issue_key=PROJ-42&owner=owner&repo=repo\n\
             - worktree://open?gitlab_host=gitlab.com&owner=owner&repo=repo&issue=42\n\
             - owner/repo#42\n\
             - owner/repo@<linear-uuid>\n\
             - org/project/repo!42\n\
             - centy:<number>\n\
             - owner/repo (ad-hoc with random branch)\n\
             - gh:<number>\n\
             - gl:<number>"
        )
    }
}
