//! Display-number-based lookup for generic items.
use super::helpers::type_storage_path;
use crate::item::core::error::ItemError;
use mdstore::{Filters, TypeConfig};
use std::path::Path;

/// Get a single generic item by display number.
pub async fn generic_get_by_display_number(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    display_number: u32,
) -> Result<mdstore::Item, ItemError> {
    if !config.features.display_number {
        return Err(ItemError::FeatureNotEnabled(
            "display_number is not enabled for this item type".to_string(),
        ));
    }
    let type_dir = type_storage_path(project_path, folder);
    let items = mdstore::list(&type_dir, Filters::new().include_deleted()).await?;
    for item in items {
        if item.frontmatter.display_number == Some(display_number) {
            return Ok(item);
        }
    }
    Err(ItemError::NotFound(format!(
        "display_number {display_number}"
    )))
}
