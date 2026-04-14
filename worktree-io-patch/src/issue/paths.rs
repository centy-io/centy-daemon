use std::path::PathBuf;

use crate::config::Config;

use super::IssueRef;

impl IssueRef {
    /// Path to the worktree checkout.
    ///
    /// For `Local`: `~/worktrees/local/{project_name}/issue-{display_number}`
    /// For others:  `~/worktrees/github/{owner}/{repo}/issue-N`
    #[must_use]
    pub fn temp_path(&self) -> PathBuf {
        self.bare_clone_path().join(self.workspace_dir_name())
    }

    /// Path to the bare clone (or the local repo itself for `Local`).
    ///
    /// # Panics
    ///
    /// Panics if the home directory cannot be determined.
    #[must_use]
    pub fn bare_clone_path(&self) -> PathBuf {
        let temp = Config::load().map(|c| c.workspace.temp).unwrap_or(false);
        self.bare_clone_path_rooted(temp)
    }

    pub(crate) fn bare_clone_path_rooted(&self, temp: bool) -> PathBuf {
        let base = if temp {
            std::env::temp_dir()
        } else {
            dirs::home_dir().expect("could not determine home directory")
        }
        .join("worktrees");
        match self {
            Self::GitHub { owner, repo, .. }
            | Self::Linear { owner, repo, .. }
            | Self::Jira { owner, repo, .. }
            | Self::Adhoc { owner, repo, .. } => base.join("github").join(owner).join(repo),
            Self::GitLab { owner, repo, .. } => base.join("gitlab").join(owner).join(repo),
            Self::AzureDevOps {
                org, project, repo, ..
            } => base.join("azuredevops").join(org).join(project).join(repo),
            Self::Local { project_path, .. } => {
                let name = project_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                base.join("local").join(name.as_ref())
            }
        }
    }
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod paths_tests;
