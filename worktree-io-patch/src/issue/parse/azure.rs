use anyhow::{bail, Context, Result};
use url::Url;

use crate::issue::IssueRef;

/// Build an [`IssueRef::AzureDevOps`] from raw `worktree://` query params.
pub(super) fn resolve_worktree_params(
    org: Option<String>,
    project: Option<String>,
    repo: Option<String>,
    id: u64,
) -> Result<IssueRef> {
    let org = org.context("Missing 'org' query param")?;
    let project = project.context("Missing 'project' query param")?;
    let repo = repo.unwrap_or_else(|| project.clone());
    Ok(IssueRef::AzureDevOps {
        org,
        project,
        repo,
        id,
    })
}

/// Parse an Azure DevOps work item URL.
///
/// Expected format: `https://dev.azure.com/{org}/{project}/_workitems/edit/{id}`
///
/// Since the URL does not include the git repository name, the project name is
/// used as the repository name by default.
pub(super) fn parse_azure_devops_url(s: &str) -> Result<IssueRef> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;

    let segments: Vec<&str> = url
        .path_segments()
        .context("URL has no path")?
        .filter(|s| !s.is_empty())
        .collect();

    // Expected: [org, project, "_workitems", "edit", id]
    if segments.len() < 5 || segments[2] != "_workitems" || segments[3] != "edit" {
        bail!(
            "Expected Azure DevOps work item URL like \
             https://dev.azure.com/org/project/_workitems/edit/42, got: {s}"
        );
    }

    let org = segments[0].to_string();
    let project = segments[1].to_string();
    let id = segments[4]
        .parse::<u64>()
        .with_context(|| format!("Invalid work item ID in URL: {}", segments[4]))?;

    Ok(IssueRef::AzureDevOps {
        repo: project.clone(),
        org,
        project,
        id,
    })
}
