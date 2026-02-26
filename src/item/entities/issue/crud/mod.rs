mod get;
mod list;
mod migrate;
mod move_issue;
mod org_sync;
mod org_sync_update;
mod parse;
mod read;
mod types;
mod update;
mod update_helpers;

pub use get::{get_issue, get_issue_by_display_number};
pub use list::{get_issues_by_uuid, list_issues};
pub use move_issue::move_issue;
pub use parse::parse_issue_md;
pub use types::{
    GetIssuesByUuidResult, IssueCrudError, IssueMetadataFlat, IssueWithProject, MoveIssueOptions,
    MoveIssueResult, UpdateIssueOptions, UpdateIssueResult,
};
pub use update::update_issue;

#[allow(deprecated)]
pub use types::Issue;

#[cfg(test)]
#[path = "../crud_tests.rs"]
mod crud_tests;
