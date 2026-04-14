use anyhow::Result;

use crate::issue::IssueRef;

pub(super) fn try_parse_shorthand(s: &str) -> Option<Result<IssueRef>> {
    // Azure DevOps: org/project/repo!42
    if let Some((path_part, id_str)) = s.split_once('!') {
        let mut parts = path_part.splitn(3, '/');
        let org = parts.next().unwrap_or("");
        let project = parts.next().unwrap_or("");
        let repo = parts.next().unwrap_or("");
        if org.is_empty() || project.is_empty() || repo.is_empty() {
            return Some(Err(anyhow::anyhow!(
                "Invalid Azure DevOps shorthand format: {s}"
            )));
        }
        let Ok(id) = id_str.parse::<u64>() else {
            return Some(Err(anyhow::anyhow!(
                "Invalid work item ID in shorthand: {id_str}"
            )));
        };
        return Some(Ok(IssueRef::AzureDevOps {
            org: org.to_string(),
            project: project.to_string(),
            repo: repo.to_string(),
            id,
        }));
    }

    if let Some((repo_part, id)) = s.split_once('@') {
        let (owner, repo) = repo_part.split_once('/')?;
        if owner.is_empty() || repo.is_empty() {
            return Some(Err(anyhow::anyhow!("Invalid shorthand format: {s}")));
        }
        if uuid::Uuid::parse_str(id).is_err() {
            return Some(Err(anyhow::anyhow!(
                "Invalid Linear issue UUID in shorthand: {id}"
            )));
        }
        return Some(Ok(IssueRef::Linear {
            owner: owner.to_string(),
            repo: repo.to_string(),
            id: id.to_string(),
        }));
    }

    let (repo_part, num_str) = s.split_once('#')?;
    let (owner, repo) = repo_part.split_once('/')?;

    if owner.is_empty() || repo.is_empty() {
        return Some(Err(anyhow::anyhow!("Invalid shorthand format: {s}")));
    }

    let Ok(number) = num_str.parse::<u64>() else {
        return Some(Err(anyhow::anyhow!(
            "Invalid issue number in shorthand: {num_str}"
        )));
    };

    Some(Ok(IssueRef::GitHub {
        owner: owner.to_string(),
        repo: repo.to_string(),
        number,
    }))
}
