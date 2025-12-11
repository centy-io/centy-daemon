pub mod assets;
pub mod create;
pub mod crud;
pub mod id;
mod metadata;
pub mod priority;
pub mod reconcile;
pub mod status;

#[allow(deprecated)]
#[allow(unused_imports)]
pub use create::{create_issue, CreateIssueOptions, CreateIssueResult};
#[allow(unused_imports)]
pub use crud::{
    delete_issue, get_issue, get_issue_by_display_number, get_issues_by_uuid, list_issues, update_issue,
    DeleteIssueResult, GetIssuesByUuidResult, Issue, IssueCrudError, IssueWithProject, UpdateIssueOptions, UpdateIssueResult,
};
#[allow(unused_imports)]
pub use metadata::IssueMetadata;
#[allow(unused_imports)]
pub use crud::IssueMetadataFlat;
pub use priority::priority_label;
#[allow(unused_imports)]
pub use assets::{
    add_asset, delete_asset, get_asset, list_assets, list_shared_assets, AssetError, AssetInfo, AssetScope,
};
#[allow(unused_imports)]
pub use id::is_uuid;
#[allow(unused_imports)]
pub use create::IssueError;
