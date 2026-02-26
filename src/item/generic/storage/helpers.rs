//! Shared helpers for generic storage operations.
use crate::item::core::error::ItemError;
use crate::manifest;
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;
/// Get the storage directory for a given item type.
pub fn type_storage_path(project_path: &Path, folder: &str) -> std::path::PathBuf {
    get_centy_path(project_path).join(folder)
}
/// Recursively copy the contents of one directory to another.
#[allow(unknown_lints, max_nesting_depth)]
pub async fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), ItemError> {
    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type().await?.is_dir() {
            fs::create_dir_all(&dst_path).await?;
            Box::pin(copy_dir_contents(&src_path, &dst_path)).await?;
        } else {
            fs::copy(&src_path, &dst_path).await?;
        }
    }
    Ok(())
}
/// Helper to update the project manifest timestamp.
pub async fn update_project_manifest(project_path: &Path) -> Result<(), ItemError> {
    if let Some(mut m) = manifest::read_manifest(project_path).await? {
        manifest::update_manifest(&mut m);
        manifest::write_manifest(project_path, &m).await?;
    }
    Ok(())
}
