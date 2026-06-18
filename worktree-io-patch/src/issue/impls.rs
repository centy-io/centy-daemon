use super::IssueRef;

impl IssueRef {
    /// Directory name used inside the bare clone for this worktree.
    #[must_use]
    pub fn workspace_dir_name(&self) -> String {
        match self {
            Self::GitHub { number, .. } | Self::GitLab { number, .. } => {
                format!("issue-{number}")
            }
            Self::Adhoc { name, .. } => name.clone(),
            Self::Linear { id, .. } => format!("linear-{id}"),
            Self::AzureDevOps { id, .. } => format!("workitem-{id}"),
            Self::Jira { issue_key, .. } => format!("jira-{}", issue_key.to_lowercase()),
            Self::Local { display_number, .. } => format!("issue-{display_number}"),
        }
    }

    /// Git branch name for this issue worktree.
    #[must_use]
    pub fn branch_name(&self) -> String {
        self.workspace_dir_name()
    }

    /// HTTPS clone URL for the repository.
    ///
    /// # Panics
    ///
    /// Always panics for `IssueRef::Local` — local repos are never cloned.
    #[must_use]
    pub fn clone_url(&self) -> String {
        match self {
            Self::GitHub { owner, repo, .. }
            | Self::Linear { owner, repo, .. }
            | Self::Jira { owner, repo, .. }
            | Self::Adhoc { owner, repo, .. } => {
                format!("https://github.com/{owner}/{repo}.git")
            }
            Self::GitLab { owner, repo, .. } => {
                format!("https://gitlab.com/{owner}/{repo}.git")
            }
            Self::AzureDevOps {
                org, project, repo, ..
            } => {
                format!("https://dev.azure.com/{org}/{project}/_git/{repo}")
            }
            Self::Local { .. } => {
                unreachable!("clone_url is never called for IssueRef::Local")
            }
        }
    }

    /// Subdirectory name within a multi-workspace root: `<repo>-<id>`.
    ///
    /// For example, `GitHub { repo: "backend", number: 7 }` → `"backend-7"`.
    ///
    /// # Panics
    ///
    /// Panics for `IssueRef::Local` — local issues are not supported in
    /// multi-workspace mode.
    #[must_use]
    pub fn multi_dir_name(&self) -> String {
        match self {
            Self::GitHub { repo, number, .. } | Self::GitLab { repo, number, .. } => {
                format!("{repo}-{number}")
            }
            Self::Adhoc { repo, name, .. } => format!("{repo}-{name}"),
            Self::Linear { repo, id, .. } => format!("{repo}-{id}"),
            Self::AzureDevOps { repo, id, .. } => format!("{repo}-{id}"),
            Self::Jira {
                repo, issue_key, ..
            } => {
                format!("{repo}-{}", issue_key.to_lowercase())
            }
            Self::Local { .. } => {
                unreachable!("multi_dir_name is not supported for IssueRef::Local")
            }
        }
    }
}
