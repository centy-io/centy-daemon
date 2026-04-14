use crate::issue::{DeepLinkOptions, IssueRef};
use anyhow::{bail, Context, Result};
use url::Url;

pub(super) fn parse_worktree_url(s: &str) -> Result<(IssueRef, DeepLinkOptions)> {
    let url = Url::parse(s).with_context(|| format!("Invalid URL: {s}"))?;
    let mut owner = None;
    let mut repo = None;
    let mut issue_num = None;
    let mut linear_id = None;
    let mut url_param = None;
    let mut editor = None;
    let mut no_hooks = false;
    let mut ado_org = None;
    let mut ado_project = None;
    let mut ado_repo = None;
    let mut ado_work_item_id = None;
    let mut jira_host = None;
    let mut jira_issue_key = None;
    let mut gitlab_host: Option<String> = None;
    for (key, val) in url.query_pairs() {
        match key.as_ref() {
            "owner" => owner = Some(val.into_owned()),
            "repo" => repo = Some(val.into_owned()),
            "issue" => {
                issue_num = Some(
                    val.parse::<u64>()
                        .with_context(|| format!("Invalid issue number: {val}"))?,
                );
            }
            "linear_id" => {
                let id = val.into_owned();
                if uuid::Uuid::parse_str(&id).is_err() {
                    bail!("Invalid Linear issue UUID: {id}");
                }
                linear_id = Some(id);
            }
            "url" => url_param = Some(val.into_owned()),
            "editor" => editor = Some(val.into_owned()),
            "no_hooks" => no_hooks = val == "1",
            "org" => ado_org = Some(val.into_owned()),
            "project" => ado_project = Some(val.into_owned()),
            "ado_repo" => ado_repo = Some(val.into_owned()),
            "work_item_id" => {
                ado_work_item_id = Some(
                    val.parse::<u64>()
                        .with_context(|| format!("Invalid work item ID: {val}"))?,
                );
            }
            "jira_host" => jira_host = Some(val.into_owned()),
            "jira_issue_key" => jira_issue_key = Some(val.into_owned()),
            "gitlab_host" => gitlab_host = Some(val.into_owned()),
            _ => {}
        }
    }
    let opts = DeepLinkOptions { editor, no_hooks };
    if let Some(url_str) = url_param {
        return Ok((super::github::parse_github_url(&url_str)?, opts));
    }
    if let Some(id) = linear_id {
        let owner = owner.context("Missing 'owner' query param")?;
        let repo = repo.context("Missing 'repo' query param")?;
        return Ok((IssueRef::Linear { owner, repo, id }, opts));
    }
    if let Some(id) = ado_work_item_id {
        return Ok((
            super::azure::resolve_worktree_params(ado_org, ado_project, ado_repo, id)?,
            opts,
        ));
    }
    if let Some(issue_key) = jira_issue_key {
        return Ok((
            super::jira::resolve_worktree_params(jira_host, issue_key, owner, repo)?,
            opts,
        ));
    }
    if gitlab_host.is_some() {
        return Ok((
            IssueRef::GitLab {
                owner: owner.context("Missing 'owner' query param")?,
                repo: repo.context("Missing 'repo' query param")?,
                number: issue_num.context("Missing 'issue' query param")?,
            },
            opts,
        ));
    }
    let owner = owner.context("Missing 'owner' query param")?;
    let repo = repo.context("Missing 'repo' query param")?;
    let issue = if let Some(number) = issue_num {
        IssueRef::GitHub {
            owner,
            repo,
            number,
        }
    } else {
        let name = crate::name_gen::generate_name();
        IssueRef::Adhoc { owner, repo, name }
    };
    Ok((issue, opts))
}
