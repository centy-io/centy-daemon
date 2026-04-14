use anyhow::{bail, Context, Result};

use crate::issue::IssueRef;

impl IssueRef {
    /// Detect the repository from the current working directory.
    ///
    /// Reads the `origin` remote URL and creates an [`Self::Adhoc`] with
    /// a randomly generated branch name.
    ///
    /// # Errors
    ///
    /// Returns an error if the current directory is not inside a git
    /// repository, has no `origin` remote, or the remote URL is not a
    /// recognised GitHub or GitLab URL.
    pub fn from_current_repo() -> Result<Self> {
        // LLVM_COV_EXCL_START
        let cwd = std::env::current_dir().context("Could not determine current directory")?;
        let remote_url = crate::git::get_remote_url(&cwd, "origin").context(
            "Not inside a git repository with an 'origin' remote.\n\
                 Run `worktree open <REF>` with an explicit issue reference.",
        )?;
        let name = crate::name_gen::generate_name();

        if let Some((owner, repo)) = super::gh::parse_github_remote_url(&remote_url) {
            return Ok(Self::Adhoc { owner, repo, name });
        }
        if let Some((owner, repo)) = super::gitlab::parse_gitlab_remote_url(&remote_url) {
            return Ok(Self::Adhoc { owner, repo, name });
        }
        bail!(
            "Remote URL {remote_url:?} is not a supported GitHub or GitLab URL.\n\
             Run `worktree open <REF>` with an explicit issue reference."
        );
        // LLVM_COV_EXCL_STOP
    }
}
