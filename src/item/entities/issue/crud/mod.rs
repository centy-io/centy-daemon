mod extra_types;
mod get;
mod get_matchers;
mod list;
mod migrate;
mod move_io;
mod move_issue;
mod org_sync;
mod org_sync_update;
mod parse;
mod read;
mod types;
mod update;
mod update_builders;
mod update_helpers;

pub use extra_types::{GetIssuesByUuidResult, IssueWithProject, MoveIssueOptions, MoveIssueResult};
pub use get::{get_issue, get_issue_by_display_number};
pub use list::{get_issues_by_uuid, list_issues};
pub use move_issue::move_issue;
pub use parse::parse_issue_md;
pub use types::Issue;
pub use types::{IssueCrudError, IssueMetadataFlat, UpdateIssueOptions, UpdateIssueResult};
pub use update::update_issue;

#[cfg(test)]
pub use read::{read_issue_from_frontmatter, read_issue_from_legacy_folder};
#[cfg(test)]
pub use update_helpers::{compute_sync_results, resolve_update_options};

#[cfg(test)]
#[path = "../crud_tests.rs"]
mod crud_tests;
