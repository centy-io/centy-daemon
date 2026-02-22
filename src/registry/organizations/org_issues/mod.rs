//! Organization-level issue management.
//!
//! Org issues are stored at ~/.centy/orgs/{slug}/issues/{uuid}.md and are
//! independent of any specific project. They can reference multiple projects.

mod config;
mod crud;
mod paths;

pub use config::{get_org_config, update_org_config, OrgConfigError, OrgCustomFieldDef};
pub use crud::{
    create_org_issue, delete_org_issue, get_org_issue, get_org_issue_by_display_number,
    list_org_issues, update_org_issue, ListOrgIssuesOptions, OrgIssue, OrgIssueError,
    UpdateOrgIssueOptions,
};
