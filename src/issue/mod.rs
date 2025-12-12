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
    delete_issue, duplicate_issue, get_issue, get_issue_by_display_number, get_issues_by_uuid,
    list_issues, move_issue, update_issue,
    DeleteIssueResult, DuplicateIssueOptions, DuplicateIssueResult, GetIssuesByUuidResult,
    Issue, IssueCrudError, IssueWithProject, MoveIssueOptions, MoveIssueResult,
    UpdateIssueOptions, UpdateIssueResult,
};
#[allow(unused_imports)]
pub use metadata::IssueMetadata;
#[allow(unused_imports)]
pub use crud::IssueMetadataFlat;
pub use priority::priority_label;
#[allow(unused_imports)]
pub use assets::{
    add_asset, copy_assets_folder, delete_asset, get_asset, list_assets, list_shared_assets,
    AssetError, AssetInfo, AssetScope,
};
#[allow(unused_imports)]
pub use id::is_uuid;
#[allow(unused_imports)]
pub use create::IssueError;
