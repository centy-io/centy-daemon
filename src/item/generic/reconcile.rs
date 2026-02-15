//! Generic display number reconciliation for resolving conflicts.
//!
//! When multiple users create items offline, they may assign the same display
//! number. This module detects and resolves such conflicts by:
//! 1. Keeping the oldest item's display number (by `created_at`)
//! 2. Reassigning newer items to the next available number

use crate::common::frontmatter::{generate_frontmatter, parse_frontmatter};
use crate::item::core::error::ItemError;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use super::types::GenericFrontmatter;

/// Information about an item needed for reconciliation.
#[derive(Debug, Clone)]
struct ItemInfo {
    /// Item ID (from filename without .md)
    id: String,
    display_number: u32,
    created_at: String,
}

/// Check if a filename is a valid item `.md` file (not `config.yaml` or other files).
fn is_item_file(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

/// Get the next available display number for a generic item type.
///
/// Scans all `*.md` files in the type storage directory and returns max + 1.
pub async fn get_next_display_number_generic(type_storage_path: &Path) -> Result<u32, ItemError> {
    if !type_storage_path.exists() {
        return Ok(1);
    }

    let mut max_number: u32 = 0;
    let mut entries = fs::read_dir(type_storage_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        if !is_item_file(&name) {
            continue;
        }

        if let Ok(content) = fs::read_to_string(entry.path()).await {
            if let Ok((frontmatter, _, _)) = parse_frontmatter::<GenericFrontmatter>(&content) {
                if let Some(dn) = frontmatter.display_number {
                    max_number = max_number.max(dn);
                }
            }
        }
    }

    Ok(max_number.saturating_add(1))
}

/// Reconcile display numbers to resolve conflicts in a generic item type.
///
/// Scans all items, finds duplicate display numbers, and reassigns them so
/// each item has a unique display number. The oldest item (by `created_at`)
/// keeps its original number.
///
/// Returns the number of items that were reassigned.
pub async fn reconcile_display_numbers_generic(type_storage_path: &Path) -> Result<u32, ItemError> {
    if !type_storage_path.exists() {
        return Ok(0);
    }

    // Step 1: Read all items and their display numbers
    let mut items: Vec<ItemInfo> = Vec::new();
    let mut entries = fs::read_dir(type_storage_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        if !is_item_file(&name) {
            continue;
        }

        let content = match fs::read_to_string(entry.path()).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        let frontmatter = match parse_frontmatter::<GenericFrontmatter>(&content) {
            Ok((fm, _, _)) => fm,
            Err(_) => continue,
        };

        if let Some(dn) = frontmatter.display_number {
            let item_id = name.trim_end_matches(".md").to_string();
            items.push(ItemInfo {
                id: item_id,
                display_number: dn,
                created_at: frontmatter.created_at,
            });
        }
    }

    // Step 2: Find duplicates (group by display_number)
    let mut by_display_number: HashMap<u32, Vec<&ItemInfo>> = HashMap::new();
    for item in &items {
        by_display_number
            .entry(item.display_number)
            .or_default()
            .push(item);
    }

    // Step 3: Find max display number for reassignment
    let max_display_number = items.iter().map(|i| i.display_number).max().unwrap_or(0);

    // Step 4: Process duplicates
    let mut reassignments: Vec<(ItemInfo, u32)> = Vec::new();
    let mut next_available = max_display_number.saturating_add(1);

    for (display_number, mut group) in by_display_number {
        if group.len() <= 1 {
            continue;
        }

        // Skip display_number 0 (items without display numbers)
        if display_number == 0 {
            for item in &group {
                reassignments.push(((*item).clone(), next_available));
                next_available = next_available.saturating_add(1);
            }
            continue;
        }

        // Sort by created_at (oldest first)
        group.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Keep the first (oldest), reassign the rest
        for item in group.iter().skip(1) {
            reassignments.push(((*item).clone(), next_available));
            next_available = next_available.saturating_add(1);
        }
    }

    // Step 5: Write reassignments
    let reassignment_count = reassignments.len() as u32;

    for (item_info, new_display_number) in reassignments {
        let file_path = type_storage_path.join(format!("{}.md", item_info.id));
        let content = fs::read_to_string(&file_path).await?;
        let (mut frontmatter, title, body): (GenericFrontmatter, String, String) =
            parse_frontmatter(&content)
                .map_err(|e| ItemError::Custom(format!("Frontmatter error: {e}")))?;
        frontmatter.display_number = Some(new_display_number);
        frontmatter.updated_at = crate::utils::now_iso();
        let new_content = generate_frontmatter(&frontmatter, &title, &body);
        fs::write(&file_path, new_content).await?;
    }

    Ok(reassignment_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::frontmatter::generate_frontmatter;
    use tempfile::TempDir;

    async fn create_test_item(
        storage_path: &Path,
        id: &str,
        display_number: u32,
        created_at: &str,
    ) {
        let frontmatter = GenericFrontmatter {
            display_number: Some(display_number),
            status: Some("open".to_string()),
            priority: Some(2),
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            deleted_at: None,
            custom_fields: HashMap::new(),
            extra: HashMap::new(),
        };

        let content = generate_frontmatter(&frontmatter, &format!("Item {id}"), "");
        fs::write(storage_path.join(format!("{id}.md")), content)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_next_display_number_empty() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        let next = get_next_display_number_generic(&storage_path)
            .await
            .unwrap();
        assert_eq!(next, 1);
    }

    #[tokio::test]
    async fn test_get_next_display_number_with_existing() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        fs::create_dir_all(&storage_path).await.unwrap();

        create_test_item(&storage_path, "item-1", 5, "2024-01-01T10:00:00Z").await;

        let next = get_next_display_number_generic(&storage_path)
            .await
            .unwrap();
        assert_eq!(next, 6);
    }

    #[tokio::test]
    async fn test_reconcile_no_conflicts() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        fs::create_dir_all(&storage_path).await.unwrap();

        create_test_item(&storage_path, "item-1", 1, "2024-01-01T10:00:00Z").await;
        create_test_item(&storage_path, "item-2", 2, "2024-01-01T11:00:00Z").await;

        let reassigned = reconcile_display_numbers_generic(&storage_path)
            .await
            .unwrap();
        assert_eq!(reassigned, 0);
    }

    #[tokio::test]
    async fn test_reconcile_with_conflict() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        fs::create_dir_all(&storage_path).await.unwrap();

        // Both have display_number 4, but different created_at
        create_test_item(&storage_path, "item-1", 4, "2024-01-01T10:00:00Z").await;
        create_test_item(&storage_path, "item-2", 4, "2024-01-01T10:05:00Z").await;
        create_test_item(&storage_path, "item-3", 5, "2024-01-01T10:10:00Z").await;

        let reassigned = reconcile_display_numbers_generic(&storage_path)
            .await
            .unwrap();
        assert_eq!(reassigned, 1);

        // Verify the older one kept display_number 4
        let content = fs::read_to_string(storage_path.join("item-1.md"))
            .await
            .unwrap();
        let (fm, _, _) = parse_frontmatter::<GenericFrontmatter>(&content).unwrap();
        assert_eq!(fm.display_number, Some(4));

        // Verify the newer one was reassigned to 6 (max was 5, so next is 6)
        let content = fs::read_to_string(storage_path.join("item-2.md"))
            .await
            .unwrap();
        let (fm, _, _) = parse_frontmatter::<GenericFrontmatter>(&content).unwrap();
        assert_eq!(fm.display_number, Some(6));
    }

    #[tokio::test]
    async fn test_reconcile_empty_directory() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");

        let reassigned = reconcile_display_numbers_generic(&storage_path)
            .await
            .unwrap();
        assert_eq!(reassigned, 0);
    }
}
