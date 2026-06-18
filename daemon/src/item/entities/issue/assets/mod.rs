mod add;
mod copy;
mod delete;
mod get;
mod helpers;
mod list;
mod list_shared;
mod types;

pub use add::add_asset;
pub use copy::copy_assets_folder;
pub use delete::delete_asset;
pub use get::get_asset;
pub use list::list_assets;
pub use list_shared::list_shared_assets;
pub use types::{
    compute_binary_hash, get_mime_type, sanitize_filename, AddAssetResult, AssetError, AssetInfo,
    AssetScope, DeleteAssetResult,
};

#[cfg(test)]
#[path = "../assets_delete_tests.rs"]
mod assets_delete_tests;
#[cfg(test)]
#[path = "../assets_get_tests.rs"]
mod assets_get_tests;
#[cfg(test)]
#[path = "../assets_list_shared_tests.rs"]
mod assets_list_shared_tests;
#[cfg(test)]
#[path = "../assets_list_tests.rs"]
mod assets_list_tests;
#[cfg(test)]
#[path = "../assets_tests.rs"]
mod assets_tests;
#[cfg(test)]
pub use list::scan_assets_directory;
