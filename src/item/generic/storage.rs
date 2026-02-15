//! Generic CRUD operations for config-driven item types.
//!
//! All functions take an `&ItemTypeConfig` and `project_path`, working
//! generically across any item type.

use crate::common::frontmatter::{generate_frontmatter, parse_frontmatter};
use crate::config::item_type_config::ItemTypeConfig;
use crate::item::core::crud::ItemFilters;
use crate::item::core::error::ItemError;
use crate::manifest;
use crate::utils::{get_centy_path, now_iso};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use super::reconcile::get_next_display_number_generic;
use super::types::{
    CreateGenericItemOptions, GenericFrontmatter, GenericItem, UpdateGenericItemOptions,
};

/// Get the storage directory for a given item type.
fn type_storage_path(project_path: &Path, config: &ItemTypeConfig) -> std::path::PathBuf {
    get_centy_path(project_path).join(&config.plural)
}

/// Get the file path for a specific item.
fn item_file_path(project_path: &Path, config: &ItemTypeConfig, id: &str) -> std::path::PathBuf {
    type_storage_path(project_path, config).join(format!("{id}.md"))
}

/// Check if a filename is a valid item file (`.md` but not `config.yaml`).
fn is_item_file(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

/// Validate status against the config's allowed statuses.
fn validate_status(config: &ItemTypeConfig, status: &str) -> Result<(), ItemError> {
    if !config.features.status {
        return Ok(());
    }
    if config.statuses.is_empty() {
        return Ok(());
    }
    if config
        .statuses
        .iter()
        .any(|s| s.eq_ignore_ascii_case(status))
    {
        Ok(())
    } else {
        Err(ItemError::InvalidStatus {
            status: status.to_string(),
            allowed: config.statuses.clone(),
        })
    }
}

/// Validate priority against the config's priority levels.
fn validate_priority(config: &ItemTypeConfig, priority: u32) -> Result<(), ItemError> {
    if !config.features.priority {
        return Ok(());
    }
    let max = config.priority_levels.unwrap_or(3);
    if priority < 1 || priority > max {
        return Err(ItemError::InvalidPriority { priority, max });
    }
    Ok(())
}

/// Create a new generic item.
///
/// Generates an ID based on the config's `id_strategy`, assigns a display number
/// if enabled, validates status/priority, and writes the item file.
pub async fn generic_create(
    project_path: &Path,
    config: &ItemTypeConfig,
    options: CreateGenericItemOptions,
) -> Result<GenericItem, ItemError> {
    let storage_path = type_storage_path(project_path, config);
    fs::create_dir_all(&storage_path).await?;

    // Generate ID
    let id = match &options.id {
        Some(explicit_id) => explicit_id.clone(),
        None => {
            if config.identifier == "slug" {
                let slug = slug::slugify(&options.title);
                if slug.is_empty() {
                    return Err(ItemError::ValidationError(
                        "Cannot generate slug from empty title".to_string(),
                    ));
                }
                slug
            } else {
                // Default to UUID
                uuid::Uuid::new_v4().to_string()
            }
        }
    };

    // Check for existing item
    let file_path = item_file_path(project_path, config, &id);
    if file_path.exists() {
        return Err(ItemError::AlreadyExists(id));
    }

    // Assign display number if enabled
    let display_number = if config.features.display_number {
        Some(get_next_display_number_generic(&storage_path).await?)
    } else {
        None
    };

    // Resolve and validate status
    let status = if config.features.status {
        let s = options
            .status
            .or_else(|| config.default_status.clone())
            .unwrap_or_default();
        validate_status(config, &s)?;
        Some(s)
    } else {
        None
    };

    // Resolve and validate priority
    let priority = if config.features.priority {
        let max = config.priority_levels.unwrap_or(3);
        let p = options
            .priority
            .unwrap_or_else(|| crate::item::validation::priority::default_priority(max));
        validate_priority(config, p)?;
        Some(p)
    } else {
        None
    };

    let now = now_iso();
    let frontmatter = GenericFrontmatter {
        display_number,
        status,
        priority,
        created_at: now.clone(),
        updated_at: now,
        deleted_at: None,
        custom_fields: options.custom_fields,
    };

    // Write the item file
    let content = generate_frontmatter(&frontmatter, &options.title, &options.body);
    fs::write(&file_path, &content).await?;

    // Update manifest
    update_project_manifest(project_path).await?;

    Ok(GenericItem {
        id,
        item_type: config.plural.clone(),
        title: options.title,
        body: options.body,
        frontmatter,
    })
}

/// Get a single generic item by ID.
pub async fn generic_get(
    project_path: &Path,
    config: &ItemTypeConfig,
    id: &str,
) -> Result<GenericItem, ItemError> {
    let file_path = item_file_path(project_path, config, id);

    if !file_path.exists() {
        return Err(ItemError::NotFound(id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    let (frontmatter, title, body) = parse_frontmatter::<GenericFrontmatter>(&content)
        .map_err(|e| ItemError::Custom(format!("Frontmatter error: {e}")))?;

    Ok(GenericItem {
        id: id.to_string(),
        item_type: config.plural.clone(),
        title,
        body,
        frontmatter,
    })
}

/// List generic items with optional filters.
pub async fn generic_list(
    project_path: &Path,
    config: &ItemTypeConfig,
    filters: ItemFilters,
) -> Result<Vec<GenericItem>, ItemError> {
    let storage_path = type_storage_path(project_path, config);
    if !storage_path.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    let mut entries = fs::read_dir(&storage_path).await?;

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

        let id = name.trim_end_matches(".md").to_string();
        let content = match fs::read_to_string(entry.path()).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (frontmatter, title, body) = match parse_frontmatter::<GenericFrontmatter>(&content) {
            Ok(result) => result,
            Err(_) => continue, // Skip malformed files
        };

        items.push(GenericItem {
            id,
            item_type: config.plural.clone(),
            title,
            body,
            frontmatter,
        });
    }

    // Apply filters
    let items = apply_filters(items, &filters);

    Ok(items)
}

/// Apply filters to a list of generic items.
fn apply_filters(mut items: Vec<GenericItem>, filters: &ItemFilters) -> Vec<GenericItem> {
    // Filter out soft-deleted unless include_deleted
    if !filters.include_deleted {
        items.retain(|item| item.frontmatter.deleted_at.is_none());
    }

    // Filter by status
    if let Some(ref status_filter) = filters.status {
        items.retain(|item| {
            item.frontmatter
                .status
                .as_ref()
                .is_some_and(|s| s.eq_ignore_ascii_case(status_filter))
        });
    }

    // Filter by priority
    if let Some(priority_filter) = filters.priority {
        items.retain(|item| item.frontmatter.priority == Some(priority_filter));
    }

    // Sort by display_number (if present), then by created_at
    items.sort_by(
        |a, b| match (a.frontmatter.display_number, b.frontmatter.display_number) {
            (Some(an), Some(bn)) => an.cmp(&bn),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.frontmatter.created_at.cmp(&b.frontmatter.created_at),
        },
    );

    // Apply offset
    if let Some(offset) = filters.offset {
        if offset < items.len() {
            items = items.split_off(offset);
        } else {
            items.clear();
        }
    }

    // Apply limit
    if let Some(limit) = filters.limit {
        items.truncate(limit);
    }

    items
}

/// Update an existing generic item.
pub async fn generic_update(
    project_path: &Path,
    config: &ItemTypeConfig,
    id: &str,
    options: UpdateGenericItemOptions,
) -> Result<GenericItem, ItemError> {
    let file_path = item_file_path(project_path, config, id);

    if !file_path.exists() {
        return Err(ItemError::NotFound(id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    let (mut frontmatter, current_title, current_body) =
        parse_frontmatter::<GenericFrontmatter>(&content)
            .map_err(|e| ItemError::Custom(format!("Frontmatter error: {e}")))?;

    // Check if item is soft-deleted
    if frontmatter.deleted_at.is_some() {
        return Err(ItemError::IsDeleted(id.to_string()));
    }

    // Update status if provided
    if let Some(ref new_status) = options.status {
        validate_status(config, new_status)?;
        frontmatter.status = Some(new_status.clone());
    }

    // Update priority if provided
    if let Some(new_priority) = options.priority {
        validate_priority(config, new_priority)?;
        frontmatter.priority = Some(new_priority);
    }

    // Merge custom fields
    for (key, value) in &options.custom_fields {
        frontmatter.custom_fields.insert(key.clone(), value.clone());
    }

    frontmatter.updated_at = now_iso();

    let title = options.title.unwrap_or(current_title);
    let body = options.body.unwrap_or(current_body);

    // Write updated file
    let new_content = generate_frontmatter(&frontmatter, &title, &body);
    fs::write(&file_path, &new_content).await?;

    // Update manifest
    update_project_manifest(project_path).await?;

    Ok(GenericItem {
        id: id.to_string(),
        item_type: config.plural.clone(),
        title,
        body,
        frontmatter,
    })
}

/// Delete an item (hard delete).
///
/// If `features.soft_delete` is enabled and the item is already soft-deleted,
/// this performs a hard delete. Otherwise, it directly removes the file.
pub async fn generic_delete(
    project_path: &Path,
    config: &ItemTypeConfig,
    id: &str,
    force: bool,
) -> Result<(), ItemError> {
    let file_path = item_file_path(project_path, config, id);

    if !file_path.exists() {
        return Err(ItemError::NotFound(id.to_string()));
    }

    // If soft-delete is enabled and not forcing, soft-delete instead
    if config.features.soft_delete && !force {
        // Check if already soft-deleted; if so, hard delete
        let content = fs::read_to_string(&file_path).await?;
        let (frontmatter, _, _) = parse_frontmatter::<GenericFrontmatter>(&content)
            .map_err(|e| ItemError::Custom(format!("Frontmatter error: {e}")))?;

        if frontmatter.deleted_at.is_none() {
            // Soft delete instead
            return generic_soft_delete(project_path, config, id).await;
        }
    }

    // Hard delete: remove the file
    fs::remove_file(&file_path).await?;

    // Remove assets directory if it exists
    if config.features.assets {
        let assets_path = get_centy_path(project_path)
            .join("assets")
            .join(&config.plural)
            .join(id);
        if assets_path.exists() {
            fs::remove_dir_all(&assets_path).await?;
        }
    }

    // Update manifest
    update_project_manifest(project_path).await?;

    Ok(())
}

/// Soft-delete an item by setting the `deleted_at` timestamp.
pub async fn generic_soft_delete(
    project_path: &Path,
    config: &ItemTypeConfig,
    id: &str,
) -> Result<(), ItemError> {
    if !config.features.soft_delete {
        return Err(ItemError::Custom(format!(
            "Soft delete is not enabled for type '{}'",
            config.name
        )));
    }

    let file_path = item_file_path(project_path, config, id);

    if !file_path.exists() {
        return Err(ItemError::NotFound(id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    let (mut frontmatter, title, body) = parse_frontmatter::<GenericFrontmatter>(&content)
        .map_err(|e| ItemError::Custom(format!("Frontmatter error: {e}")))?;

    if frontmatter.deleted_at.is_some() {
        return Err(ItemError::IsDeleted(id.to_string()));
    }

    frontmatter.deleted_at = Some(now_iso());
    frontmatter.updated_at = now_iso();

    let new_content = generate_frontmatter(&frontmatter, &title, &body);
    fs::write(&file_path, &new_content).await?;

    update_project_manifest(project_path).await?;

    Ok(())
}

/// Restore a soft-deleted item by clearing the `deleted_at` timestamp.
pub async fn generic_restore(
    project_path: &Path,
    config: &ItemTypeConfig,
    id: &str,
) -> Result<(), ItemError> {
    if !config.features.soft_delete {
        return Err(ItemError::Custom(format!(
            "Soft delete is not enabled for type '{}'",
            config.name
        )));
    }

    let file_path = item_file_path(project_path, config, id);

    if !file_path.exists() {
        return Err(ItemError::NotFound(id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    let (mut frontmatter, title, body) = parse_frontmatter::<GenericFrontmatter>(&content)
        .map_err(|e| ItemError::Custom(format!("Frontmatter error: {e}")))?;

    if frontmatter.deleted_at.is_none() {
        return Err(ItemError::Custom(format!("Item '{id}' is not deleted")));
    }

    frontmatter.deleted_at = None;
    frontmatter.updated_at = now_iso();

    let new_content = generate_frontmatter(&frontmatter, &title, &body);
    fs::write(&file_path, &new_content).await?;

    update_project_manifest(project_path).await?;

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
    use crate::config::item_type_config::{default_issue_config, ItemTypeFeatures};
    use crate::config::CentyConfig;

    async fn setup_project(temp: &Path) {
        let centy_path = temp.join(".centy");
        fs::create_dir_all(&centy_path).await.unwrap();

        // Write manifest
        let manifest = manifest::create_manifest();
        manifest::write_manifest(temp, &manifest).await.unwrap();
    }

    fn minimal_config() -> ItemTypeConfig {
        ItemTypeConfig {
            name: "Note".to_string(),
            plural: "notes".to_string(),
            identifier: "uuid".to_string(),
            features: ItemTypeFeatures::default(),
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

        let config = default_issue_config(&CentyConfig::default());
        let options = CreateGenericItemOptions {
            title: "Test Issue".to_string(),
            body: "This is a test.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();
        assert_eq!(created.title, "Test Issue");
        assert_eq!(created.body, "This is a test.");
        assert_eq!(created.frontmatter.display_number, Some(1));
        assert_eq!(created.frontmatter.status, Some("open".to_string()));
        assert_eq!(created.frontmatter.priority, Some(2));

        // Get it back
        let fetched = generic_get(temp.path(), &config, &created.id)
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
        let options = CreateGenericItemOptions {
            title: "Simple Note".to_string(),
            body: "Just a note.".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();
        assert!(created.frontmatter.display_number.is_none());
        assert!(created.frontmatter.status.is_none());
        assert!(created.frontmatter.priority.is_none());
    }

    #[tokio::test]
    async fn test_create_slug_id_strategy() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let mut config = minimal_config();
        config.identifier = "slug".to_string();
        config.plural = "docs".to_string();

        let options = CreateGenericItemOptions {
            title: "Getting Started Guide".to_string(),
            body: "Welcome!".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();
        assert_eq!(created.id, "getting-started-guide");
    }

    #[tokio::test]
    async fn test_create_invalid_status() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());
        let options = CreateGenericItemOptions {
            title: "Bad Status".to_string(),
            body: String::new(),
            id: None,
            status: Some("nonexistent".to_string()),
            priority: None,
            custom_fields: HashMap::new(),
        };

        let result = generic_create(temp.path(), &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::InvalidStatus { .. })));
    }

    #[tokio::test]
    async fn test_create_invalid_priority() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());
        let options = CreateGenericItemOptions {
            title: "Bad Priority".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(99),
            custom_fields: HashMap::new(),
        };

        let result = generic_create(temp.path(), &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::InvalidPriority { .. })));
    }

    #[tokio::test]
    async fn test_list_with_filters() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());

        // Create multiple items
        for (title, status) in [
            ("Open 1", "open"),
            ("Open 2", "open"),
            ("Closed 1", "closed"),
        ] {
            let options = CreateGenericItemOptions {
                title: title.to_string(),
                body: String::new(),
                id: None,
                status: Some(status.to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            };
            generic_create(temp.path(), &config, options).await.unwrap();
        }

        // List all
        let all = generic_list(temp.path(), &config, ItemFilters::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 3);

        // List open only
        let open = generic_list(temp.path(), &config, ItemFilters::new().with_status("open"))
            .await
            .unwrap();
        assert_eq!(open.len(), 2);

        // List with limit
        let limited = generic_list(temp.path(), &config, ItemFilters::new().with_limit(1))
            .await
            .unwrap();
        assert_eq!(limited.len(), 1);

        // List with offset
        let offset = generic_list(temp.path(), &config, ItemFilters::new().with_offset(2))
            .await
            .unwrap();
        assert_eq!(offset.len(), 1);
    }

    #[tokio::test]
    async fn test_update() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());
        let options = CreateGenericItemOptions {
            title: "Original Title".to_string(),
            body: "Original body.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();

        let update_options = UpdateGenericItemOptions {
            title: Some("Updated Title".to_string()),
            body: Some("Updated body.".to_string()),
            status: Some("closed".to_string()),
            priority: Some(1),
            custom_fields: HashMap::from([("env".to_string(), serde_json::json!("prod"))]),
        };

        let updated = generic_update(temp.path(), &config, &created.id, update_options)
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

        let config = default_issue_config(&CentyConfig::default());
        let result = generic_update(
            temp.path(),
            &config,
            "nonexistent",
            UpdateGenericItemOptions::default(),
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_soft_delete_and_restore() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());
        let options = CreateGenericItemOptions {
            title: "To Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();

        // Soft delete
        generic_soft_delete(temp.path(), &config, &created.id)
            .await
            .unwrap();

        // Should not appear in default list
        let items = generic_list(temp.path(), &config, ItemFilters::default())
            .await
            .unwrap();
        assert!(items.is_empty());

        // Should appear with include_deleted
        let items = generic_list(temp.path(), &config, ItemFilters::new().include_deleted())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.first().unwrap().frontmatter.deleted_at.is_some());

        // Restore
        generic_restore(temp.path(), &config, &created.id)
            .await
            .unwrap();

        // Should appear again
        let items = generic_list(temp.path(), &config, ItemFilters::default())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.first().unwrap().frontmatter.deleted_at.is_none());
    }

    #[tokio::test]
    async fn test_hard_delete() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());
        let options = CreateGenericItemOptions {
            title: "To Hard Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();

        // Force hard delete
        generic_delete(temp.path(), &config, &created.id, true)
            .await
            .unwrap();

        // Should not exist at all
        let result = generic_get(temp.path(), &config, &created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_soft_delete_not_enabled() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = minimal_config(); // soft_delete = false
        let options = CreateGenericItemOptions {
            title: "Note".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();

        let result = generic_soft_delete(temp.path(), &config, &created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_display_number_auto_increment() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());

        for i in 1..=3u32 {
            let options = CreateGenericItemOptions {
                title: format!("Issue {i}"),
                body: String::new(),
                id: None,
                status: Some("open".to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            };

            let created = generic_create(temp.path(), &config, options).await.unwrap();
            assert_eq!(created.frontmatter.display_number, Some(i));
        }
    }

    #[tokio::test]
    async fn test_update_preserves_fields() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());
        let options = CreateGenericItemOptions {
            title: "Keep Fields".to_string(),
            body: "Original body.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(1),
            custom_fields: HashMap::from([("key".to_string(), serde_json::json!("value"))]),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();

        // Update only the title
        let updated = generic_update(
            temp.path(),
            &config,
            &created.id,
            UpdateGenericItemOptions {
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

        let config = default_issue_config(&CentyConfig::default());
        let options = CreateGenericItemOptions {
            title: "Will Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = generic_create(temp.path(), &config, options).await.unwrap();
        generic_soft_delete(temp.path(), &config, &created.id)
            .await
            .unwrap();

        let result = generic_update(
            temp.path(),
            &config,
            &created.id,
            UpdateGenericItemOptions {
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
        config.identifier = "slug".to_string();

        let options = CreateGenericItemOptions {
            title: "Same Title".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        generic_create(temp.path(), &config, options.clone())
            .await
            .unwrap();

        let result = generic_create(temp.path(), &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());
        let result = generic_get(temp.path(), &config, "nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ItemError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_list_empty() {
        let temp = tempfile::tempdir().unwrap();
        setup_project(temp.path()).await;

        let config = default_issue_config(&CentyConfig::default());
        let items = generic_list(temp.path(), &config, ItemFilters::default())
            .await
            .unwrap();
        assert!(items.is_empty());
    }
}
