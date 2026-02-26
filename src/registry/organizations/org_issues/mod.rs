//! Organization-level issue management.
//!
//! Org issues are stored at ~/.centy/orgs/{slug}/issues/{uuid}.md and are
//! independent of any specific project. They can reference multiple projects.

mod config;
mod crud;
mod crud_ops;
mod crud_types;
mod paths;

pub use config::{get_org_config, update_org_config, OrgConfigError, OrgCustomFieldDef};
pub use crud::{delete_org_issue, update_org_issue, UpdateOrgIssueOptions};
pub use crud_ops::{
    create_org_issue, get_org_issue, get_org_issue_by_display_number, list_org_issues,
};
pub use crud_types::{ListOrgIssuesOptions, OrgIssue, OrgIssueError};
