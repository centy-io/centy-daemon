//! Generic CRUD operations for config-driven item types.
//!
//! Thin wrappers around `mdstore` that add daemon-specific concerns:
//! project manifest updates, asset handling, and project-path resolution.

use crate::item::core::error::ItemError;
use crate::manifest;
use crate::utils::get_centy_path;
use mdstore::{CreateOptions, Filters, TypeConfig, UpdateOptions};
use std::path::Path;
use tokio::fs;

use super::types::DuplicateGenericItemOptions;

/// Get the storage directory for a given item type.
fn type_storage_path(project_path: &Path, folder: &str) -> std::path::PathBuf {
    get_centy_path(project_path).join(folder)
}

/// Create a new generic item.
pub async fn generic_create(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    options: CreateOptions,
) -> Result<mdstore::Item, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    let item = mdstore::create(&type_dir, config, options).await?;
    update_project_manifest(project_path).await?;
    Ok(item)
}

/// Get a single generic item by ID.
pub async fn generic_get(
    project_path: &Path,
    folder: &str,
    id: &str,
) -> Result<mdstore::Item, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    Ok(mdstore::get(&type_dir, id).await?)
}

/// Get a single generic item by display number.
///
/// Lists all items via `mdstore::list` and finds the one whose frontmatter
/// `display_number` matches the requested number. Returns `FeatureNotEnabled`
/// if the item type does not have `display_number` enabled.
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

/// List generic items with optional filters.
pub async fn generic_list(
    project_path: &Path,
    folder: &str,
    filters: Filters,
) -> Result<Vec<mdstore::Item>, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    Ok(mdstore::list(&type_dir, filters).await?)
}

/// Update an existing generic item.
pub async fn generic_update(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    id: &str,
    options: UpdateOptions,
) -> Result<mdstore::Item, ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    let item = mdstore::update(&type_dir, config, id, options).await?;
    update_project_manifest(project_path).await?;
    Ok(item)
}

/// Delete an item (hard delete).
///
/// If the item is not yet soft-deleted and `force` is false, this performs a
/// soft delete instead. If the item is already soft-deleted (or `force` is
/// true), this removes the file permanently and cleans up assets.
pub async fn generic_delete(
    project_path: &Path,
    folder: &str,
    config: &TypeConfig,
    id: &str,
    force: bool,
) -> Result<(), ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    mdstore::delete(&type_dir, id, force).await?;

    // Remove assets directory if it exists (only on hard delete)
    if force && config.features.assets {
        let assets_path = get_centy_path(project_path)
            .join("assets")
            .join(folder)
            .join(id);
        if assets_path.exists() {
            fs::remove_dir_all(&assets_path).await?;
        }
    }

    update_project_manifest(project_path).await?;
    Ok(())
}

/// Soft-delete an item by setting the `deleted_at` timestamp.
pub async fn generic_soft_delete(
    project_path: &Path,
    folder: &str,
    id: &str,
) -> Result<(), ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    mdstore::soft_delete(&type_dir, id).await?;
    update_project_manifest(project_path).await?;
    Ok(())
}

/// Restore a soft-deleted item by clearing the `deleted_at` timestamp.
pub async fn generic_restore(project_path: &Path, folder: &str, id: &str) -> Result<(), ItemError> {
    let type_dir = type_storage_path(project_path, folder);
    mdstore::restore(&type_dir, id).await?;
    update_project_manifest(project_path).await?;
    Ok(())
}

/// Duplicate an item to the same or different project.
///
/// Converts daemon-specific `DuplicateGenericItemOptions` (project paths) to
/// mdstore `DuplicateOptions` (type directories), delegates to mdstore, then
/// handles asset copying and manifest updates.
pub async fn generic_duplicate(
    folder: &str,
    config: &TypeConfig,
    options: DuplicateGenericItemOptions,
) -> Result<mdstore::DuplicateResult, ItemError> {
    let source_dir = type_storage_path(&options.source_project_path, folder);
    let target_dir = type_storage_path(&options.target_project_path, folder);

    let mdstore_options = mdstore::DuplicateOptions {
        source_dir: source_dir.clone(),
        target_dir: target_dir.clone(),
        item_id: options.item_id.clone(),
        new_id: options.new_id,
        new_title: options.new_title,
    };

    let result = mdstore::duplicate(config, mdstore_options).await?;

    // Copy assets if enabled
    if config.features.assets {
        let source_assets = source_dir.join("assets").join(&options.item_id);
        let target_assets = target_dir.join("assets").join(&result.item.id);
        if source_assets.exists() {
            fs::create_dir_all(&target_assets).await?;
            copy_dir_contents(&source_assets, &target_assets).await?;
        }
    }

    // Update target manifest
    update_project_manifest(&options.target_project_path).await?;

    Ok(result)
}

/// Move an item from one project to another.
///
/// Performs daemon-specific pre-checks (feature flag, manifest validation),
/// delegates file operations to mdstore, then handles asset copying/cleanup
/// and manifest updates on both projects.
pub async fn generic_move(
    source_project_path: &Path,
    target_project_path: &Path,
    source_folder: &str,
    target_folder: &str,
    source_config: &TypeConfig,
    target_config: &TypeConfig,
    item_id: &str,
    new_id: Option<&str>,
) -> Result<mdstore::MoveResult, ItemError> {
    // 1. Validate both projects initialized
    manifest::read_manifest(source_project_path)
        .await?
        .ok_or(ItemError::NotInitialized)?;
    manifest::read_manifest(target_project_path)
        .await?
        .ok_or(ItemError::TargetNotInitialized)?;

    let source_dir = type_storage_path(source_project_path, source_folder);
    let target_dir = type_storage_path(target_project_path, target_folder);

    // 2. Copy assets before the move (source file will be deleted by mdstore)
    let copied_assets = if source_config.features.assets {
        // Check new-format asset path: .centy/assets/<folder>/<id>/
        let source_assets_new = get_centy_path(source_project_path)
            .join("assets")
            .join(source_folder)
            .join(item_id);
        // Also check legacy path: .centy/<folder>/assets/<id>/
        let source_assets_legacy = source_dir.join("assets").join(item_id);

        let source_assets = if source_assets_new.exists() {
            Some(source_assets_new)
        } else if source_assets_legacy.exists() {
            Some(source_assets_legacy)
        } else {
            None
        };

        if let Some(ref src_assets) = source_assets {
            // Determine target ID for assets (use new_id if slug-based, otherwise same id)
            let target_id = if source_config.identifier == mdstore::IdStrategy::Slug {
                new_id.unwrap_or(item_id)
            } else {
                item_id
            };
            let target_assets = get_centy_path(target_project_path)
                .join("assets")
                .join(target_folder)
                .join(target_id);
            fs::create_dir_all(&target_assets).await?;
            copy_dir_contents(src_assets, &target_assets).await?;
        }

        source_assets
    } else {
        None
    };

    // 3. Delegate file move to mdstore
    let result = mdstore::move_item(
        &source_dir,
        &target_dir,
        source_config,
        target_config,
        item_id,
        new_id,
    )
    .await?;

    // 4. Clean up source assets
    if let Some(src_assets) = copied_assets {
        if src_assets.exists() {
            fs::remove_dir_all(&src_assets).await?;
        }
    }
    // Also clean up legacy path if it exists
    if source_config.features.assets {
        let source_assets_legacy = source_dir.join("assets").join(item_id);
        if source_assets_legacy.exists() {
            fs::remove_dir_all(&source_assets_legacy).await?;
        }
    }

    // 5. Update both manifests
    update_project_manifest(source_project_path).await?;
    update_project_manifest(target_project_path).await?;

    Ok(result)
}

/// Rename a slug-based item within the same project folder.
///
/// This is used when `move_item` is called with `source_path == target_path`
/// and a non-empty `new_id`.  The `mdstore::move_item` implementation rejects
/// same-directory moves, so we handle the rename directly.
pub async fn generic_rename_slug(
    project_path: &Path,
    folder: &str,
    _config: &TypeConfig,
    item_id: &str,
    new_id: &str,
) -> Result<mdstore::MoveResult, ItemError> {
    let type_dir = type_storage_path(project_path, folder);

    let source_file = type_dir.join(format!("{item_id}.md"));
    let target_file = type_dir.join(format!("{new_id}.md"));

    if !source_file.exists() {
        return Err(ItemError::NotFound(item_id.to_string()));
    }
    if target_file.exists() {
        return Err(ItemError::Custom(format!(
            "item with id '{new_id}' already exists"
        )));
    }

    // Read the item first (for the return value)
    let mut item = mdstore::get(&type_dir, item_id).await?;
    item.id = new_id.to_string();

    // Rename the file
    tokio::fs::rename(&source_file, &target_file).await?;

    // Update the manifest
    update_project_manifest(project_path).await?;

    Ok(mdstore::MoveResult {
        item,
        old_id: item_id.to_string(),
    })
}

/// Recursively copy the contents of one directory to another.
async fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), ItemError> {
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
async fn update_project_manifest(project_path: &Path) -> Result<(), ItemError> {
    if let Some(mut m) = manifest::read_manifest(project_path).await? {
        manifest::update_manifest(&mut m);
        manifest::write_manifest(project_path, &m).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::item_type_config::default_issue_config;
    use crate::config::CentyConfig;
    use std::collections::HashMap;

    async fn setup_project(temp: &Path) {
        let centy_path = temp.join(".centy");
        fs::create_dir_all(&centy_path).await.unwrap();

        // Write manifest
        let manifest = manifest::create_manifest();
        manifest::write_manifest(temp, &manifest).await.unwrap();
    }

    /// Helper: build the default issue `TypeConfig` for use with generic storage functions.
    fn issue_type_config() -> TypeConfig {
        TypeConfig::from(&default_issue_config(&CentyConfig::default()))
    }

    fn minimal_config() -> TypeConfig {
        TypeConfig {
            name: "Note".to_string(),
            identifier: mdstore::IdStrategy::Uuid,
            features: mdstore::TypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_create_and_get() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let options = CreateOptions {
            title: "Test Issue".to_string(),
            body: "This is a test.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), "issues", &config, options)
            .await
            .unwrap();
        assert_eq!(created.title, "Test Issue");
        assert_eq!(created.body, "This is a test.");
        assert_eq!(created.frontmatter.display_number, Some(1));
        assert_eq!(created.frontmatter.status, Some("open".to_string()));
        assert_eq!(created.frontmatter.priority, Some(2));

        // Get it back
        let fetched = generic_get(temp.path(), "issues", &created.id)
            .await
            .unwrap();
        assert_eq!(fetched.title, "Test Issue");
        assert_eq!(fetched.frontmatter.display_number, Some(1));
    }

    #[tokio::test]
    async fn test_create_minimal_features() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = minimal_config();
        let options = CreateOptions {
            title: "Simple Note".to_string(),
            body: "Just a note.".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), "notes", &config, options)
            .await
            .unwrap();
        assert!(created.frontmatter.display_number.is_none());
        assert!(created.frontmatter.status.is_none());
        assert!(created.frontmatter.priority.is_none());
    }

    #[tokio::test]
    async fn test_create_slug_id_strategy() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let mut config = minimal_config();
        config.identifier = mdstore::IdStrategy::Slug;

        let options = CreateOptions {
            title: "Getting Started Guide".to_string(),
            body: "Welcome!".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), "docs", &config, options)
            .await
            .unwrap();
        assert_eq!(created.id, "getting-started-guide");
    }

    #[tokio::test]
    async fn test_create_invalid_status() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let options = CreateOptions {
            title: "Bad Status".to_string(),
            body: String::new(),
            id: None,
            status: Some("nonexistent".to_string()),
            priority: None,
            custom_fields: HashMap::new(),
        };

        let result = generic_create(temp.path(), "issues", &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::InvalidStatus { .. })));
    }

    #[tokio::test]
    async fn test_create_invalid_priority() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let options = CreateOptions {
            title: "Bad Priority".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(99),
            custom_fields: HashMap::new(),
        };

        let result = generic_create(temp.path(), "issues", &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::InvalidPriority { .. })));
    }

    #[tokio::test]
    async fn test_list_with_filters() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();

        // Create multiple items
        for (title, status) in [
            ("Open 1", "open"),
            ("Open 2", "open"),
            ("Closed 1", "closed"),
        ] {
            let options = CreateOptions {
                title: title.to_string(),
                body: String::new(),
                id: None,
                status: Some(status.to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            };
            generic_create(temp.path(), "issues", &config, options)
                .await
                .unwrap();
        }

        // List all
        let all = generic_list(temp.path(), "issues", Filters::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 3);

        // List open only
        let open = generic_list(temp.path(), "issues", Filters::new().with_status("open"))
            .await
            .unwrap();
        assert_eq!(open.len(), 2);

        // List with limit
        let limited = generic_list(temp.path(), "issues", Filters::new().with_limit(1))
            .await
            .unwrap();
        assert_eq!(limited.len(), 1);

        // List with offset
        let offset = generic_list(temp.path(), "issues", Filters::new().with_offset(2))
            .await
            .unwrap();
        assert_eq!(offset.len(), 1);
    }

    #[tokio::test]
    async fn test_update() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let options = CreateOptions {
            title: "Original Title".to_string(),
            body: "Original body.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), "issues", &config, options)
            .await
            .unwrap();

        let update_options = UpdateOptions {
            title: Some("Updated Title".to_string()),
            body: Some("Updated body.".to_string()),
            status: Some("closed".to_string()),
            priority: Some(1),
            custom_fields: HashMap::from([("env".to_string(), serde_json::json!("prod"))]),
        };

        let updated = generic_update(temp.path(), "issues", &config, &created.id, update_options)
            .await
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.body, "Updated body.");
        assert_eq!(updated.frontmatter.status, Some("closed".to_string()));
        assert_eq!(updated.frontmatter.priority, Some(1));
        assert_eq!(
            updated.frontmatter.custom_fields.get("env"),
            Some(&serde_json::json!("prod"))
        );
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let result = generic_update(
            temp.path(),
            "issues",
            &config,
            "nonexistent",
            UpdateOptions::default(),
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_soft_delete_and_restore() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let options = CreateOptions {
            title: "To Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), "issues", &config, options)
            .await
            .unwrap();

        // Soft delete
        generic_soft_delete(temp.path(), "issues", &created.id)
            .await
            .unwrap();

        // Should not appear in default list
        let items = generic_list(temp.path(), "issues", Filters::default())
            .await
            .unwrap();
        assert!(items.is_empty());

        // Should appear with include_deleted
        let items = generic_list(temp.path(), "issues", Filters::new().include_deleted())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.first().unwrap().frontmatter.deleted_at.is_some());

        // Restore
        generic_restore(temp.path(), "issues", &created.id)
            .await
            .unwrap();

        // Should appear again
        let items = generic_list(temp.path(), "issues", Filters::default())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.first().unwrap().frontmatter.deleted_at.is_none());
    }

    #[tokio::test]
    async fn test_hard_delete() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let options = CreateOptions {
            title: "To Hard Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), "issues", &config, options)
            .await
            .unwrap();

        // Force hard delete
        generic_delete(temp.path(), "issues", &config, &created.id, true)
            .await
            .unwrap();

        // Should not exist at all
        let result = generic_get(temp.path(), "issues", &created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_display_number_auto_increment() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();

        for i in 1..=3u32 {
            let options = CreateOptions {
                title: format!("Issue {i}"),
                body: String::new(),
                id: None,
                status: Some("open".to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            };

            let created = generic_create(temp.path(), "issues", &config, options)
                .await
                .unwrap();
            assert_eq!(created.frontmatter.display_number, Some(i));
        }
    }

    #[tokio::test]
    async fn test_update_preserves_fields() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let options = CreateOptions {
            title: "Keep Fields".to_string(),
            body: "Original body.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(1),
            custom_fields: HashMap::from([("key".to_string(), serde_json::json!("value"))]),
        };

        let created = generic_create(temp.path(), "issues", &config, options)
            .await
            .unwrap();

        // Update only the title
        let updated = generic_update(
            temp.path(),
            "issues",
            &config,
            &created.id,
            UpdateOptions {
                title: Some("New Title".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.body, "Original body.");
        assert_eq!(updated.frontmatter.status, Some("open".to_string()));
        assert_eq!(updated.frontmatter.priority, Some(1));
        assert_eq!(
            updated.frontmatter.custom_fields.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[tokio::test]
    async fn test_cannot_update_deleted_item() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();
        let options = CreateOptions {
            title: "Will Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), "issues", &config, options)
            .await
            .unwrap();
        generic_soft_delete(temp.path(), "issues", &created.id)
            .await
            .unwrap();

        let result = generic_update(
            temp.path(),
            "issues",
            &config,
            &created.id,
            UpdateOptions {
                title: Some("Fail".to_string()),
                ..Default::default()
            },
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::IsDeleted(_))));
    }

    #[tokio::test]
    async fn test_already_exists() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let mut config = minimal_config();
        config.identifier = mdstore::IdStrategy::Slug;

        let options = CreateOptions {
            title: "Same Title".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        generic_create(temp.path(), "notes", &config, options.clone())
            .await
            .unwrap();

        let result = generic_create(temp.path(), "notes", &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let result = generic_get(temp.path(), "issues", "nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_list_empty() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let items = generic_list(temp.path(), "issues", Filters::default())
            .await
            .unwrap();
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_get_by_display_number_success() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();

        // Create two items
        let options1 = CreateOptions {
            title: "First Issue".to_string(),
            body: "Body 1".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };
        let created1 = generic_create(temp.path(), "issues", &config, options1)
            .await
            .unwrap();
        assert_eq!(created1.frontmatter.display_number, Some(1));

        let options2 = CreateOptions {
            title: "Second Issue".to_string(),
            body: "Body 2".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(1),
            custom_fields: HashMap::new(),
        };
        let created2 = generic_create(temp.path(), "issues", &config, options2)
            .await
            .unwrap();
        assert_eq!(created2.frontmatter.display_number, Some(2));

        // Look up by display number
        let found = generic_get_by_display_number(temp.path(), "issues", &config, 2)
            .await
            .unwrap();
        assert_eq!(found.title, "Second Issue");
        assert_eq!(found.id, created2.id);
    }

    #[tokio::test]
    async fn test_get_by_display_number_not_found() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = issue_type_config();

        let options = CreateOptions {
            title: "Only Issue".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };
        generic_create(temp.path(), "issues", &config, options)
            .await
            .unwrap();

        let result = generic_get_by_display_number(temp.path(), "issues", &config, 99).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_get_by_display_number_feature_disabled() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = minimal_config(); // display_number feature is disabled
        let result = generic_get_by_display_number(temp.path(), "notes", &config, 1).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::FeatureNotEnabled(_))));
    }
}
