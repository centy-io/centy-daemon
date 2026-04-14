use std::path::PathBuf;

/// A reference to an issue that identifies a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueRef {
    /// A GitHub issue identified by owner, repo, and number.
    GitHub {
        /// GitHub organization or user name.
        owner: String,
        /// Repository name.
        repo: String,
        /// Issue number.
        number: u64,
    },
    /// A Linear issue identified by its UUID, paired with the GitHub repo that
    /// hosts the code for that project.
    Linear {
        /// GitHub organization or user name that hosts the code.
        owner: String,
        /// Repository name.
        repo: String,
        /// Linear issue UUID.
        id: String,
    },
    /// An Azure DevOps work item paired with an Azure Repos git repository.
    AzureDevOps {
        /// Azure DevOps organization name.
        org: String,
        /// Azure DevOps project name.
        project: String,
        /// Azure Repos git repository name.
        repo: String,
        /// Work item ID.
        id: u64,
    },
    /// A Jira issue paired with a GitHub repo that hosts the code.
    Jira {
        /// Jira instance host (e.g. `acme.atlassian.net`).
        host: String,
        /// Jira issue key (e.g. `PROJ-42`).
        issue_key: String,
        /// GitHub organization or user name that hosts the code.
        owner: String,
        /// Repository name.
        repo: String,
    },
    /// A GitLab issue identified by owner, repo, and number.
    GitLab {
        /// GitLab group or user name.
        owner: String,
        /// Repository name.
        repo: String,
        /// Issue number.
        number: u64,
    },
    /// A bare repo opened without a specific issue — random branch name.
    Adhoc {
        /// GitHub organization or user name.
        owner: String,
        /// Repository name.
        repo: String,
        /// Auto-generated name (e.g. `bold_turing`).
        name: String,
    },
    /// A local Centy issue — the repository itself is the source, no remote clone needed.
    Local {
        /// Absolute path to the local project repository.
        project_path: PathBuf,
        /// Human-readable issue number shown in the branch name.
        display_number: u32,
    },
}
