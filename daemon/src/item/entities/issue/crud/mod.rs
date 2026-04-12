mod extra_types;
mod get;
mod get_matchers;
mod list;
mod move_io;
mod move_issue;
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
pub use read::read_issue_from_frontmatter;
#[cfg(test)]
pub use update_helpers::resolve_update_options;

#[cfg(test)]
#[path = "../crud_tests.rs"]
mod crud_tests;
