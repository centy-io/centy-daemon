use anyhow::Result;

use crate::issue::{DeepLinkOptions, IssueRef};

use super::worktree_url;

impl IssueRef {
    /// Like [`parse`] but also returns any [`DeepLinkOptions`] embedded in a
    /// `worktree://` URL (e.g. the `editor` query param).
    ///
    /// # Errors
    ///
    /// Returns an error if `s` cannot be parsed as a valid issue reference.
    pub fn parse_with_options(s: &str) -> Result<(Self, DeepLinkOptions)> {
        let s = s.trim();
        if s.starts_with("worktree://") {
            return worktree_url::parse_worktree_url(s);
        }
        Ok((Self::parse(s)?, DeepLinkOptions::default()))
    }
}
