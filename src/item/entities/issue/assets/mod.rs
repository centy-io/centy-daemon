mod add;
mod copy;
mod delete;
mod get;
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
#[path = "../assets_tests.rs"]
mod assets_tests;
