mod compat;
mod create_fn;
mod helpers;
mod render;
mod types;
mod write_issue;

#[allow(deprecated)]
pub use compat::{create_issue_with_title_generation, get_next_issue_number};
pub use create_fn::create_issue;
pub use types::{CreateIssueOptions, CreateIssueResult, IssueError};
