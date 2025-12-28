use std::collections::HashMap;

use crate::issue::metadata::ImportMetadata;
use crate::utils::now_iso;
use super::config::{FieldMappings, ProviderConfig};
use super::error::MapperError;
use super::provider::ExternalTask;

/// Map external task to Centy issue creation options
///
/// Returns a tuple of (title, description, status, priority, custom_fields, import_metadata)
pub fn map_external_task_to_create(
    task: &ExternalTask,
    config: &ProviderConfig,
) -> Result<
    (
        String,
        String,
        String,
        u32,
        HashMap<String, String>,
        ImportMetadata,
    ),
    MapperError,
> {
    let status = map_status(&task.status, &config.field_mappings);
    let priority = map_priority(task, &config.field_mappings);
    let custom_fields = map_labels_to_custom_fields(&task.labels, &config.field_mappings);

    // Build description with original metadata footer
    let description = format!(
        "{}\n\n---\n\n**Imported from:** {}\n**Original author:** {}\n**Original created:** {}",
        task.description,
        task.url,
        task.author.as_deref().unwrap_or("unknown"),
        task.created_at
    );

    let import_metadata = ImportMetadata {
        provider: config.provider.clone(),
        source_id: config.source_id.clone(),
        external_id: task.external_id.clone(),
        url: task.url.clone(),
        imported_at: now_iso(),
        last_synced_at: now_iso(),
    };

    Ok((
        task.title.clone(),
        description,
        status,
        priority,
        custom_fields,
        import_metadata,
    ))
}

/// Map external task to Centy issue update options
///
/// Returns a tuple of (title, description, status, custom_fields, import_metadata)
pub fn map_external_task_to_update(
    task: &ExternalTask,
    config: &ProviderConfig,
    existing_imported_at: String,
) -> Result<(String, String, String, HashMap<String, String>, ImportMetadata), MapperError> {
    let status = map_status(&task.status, &config.field_mappings);
    let custom_fields = map_labels_to_custom_fields(&task.labels, &config.field_mappings);

    let import_metadata = ImportMetadata {
        provider: config.provider.clone(),
        source_id: config.source_id.clone(),
        external_id: task.external_id.clone(),
        url: task.url.clone(),
        imported_at: existing_imported_at, // Keep original import time
        last_synced_at: now_iso(),
    };

    Ok((
        task.title.clone(),
        task.description.clone(),
        status,
        custom_fields,
        import_metadata,
    ))
}

/// Map external status to Centy status using configured mappings
fn map_status(external_status: &str, mappings: &FieldMappings) -> String {
    mappings
        .status_mapping
        .get(external_status)
        .cloned()
        .or_else(|| mappings.default_status.clone())
        .unwrap_or_else(|| "open".to_string())
}

/// Extract priority from labels or use default
fn map_priority(task: &ExternalTask, mappings: &FieldMappings) -> u32 {
    // Check if any label maps to priority
    for label in &task.labels {
        if let Some(mapping) = mappings.label_to_custom_field.get(label) {
            if let Some(("priority", value)) = mapping.split_once('=') {
                if let Ok(priority) = value.parse::<u32>() {
                    return priority;
                }
            }
        }
    }

    // Fallback to default
    mappings.default_priority.unwrap_or(3)
}

/// Map external labels to Centy custom fields
fn map_labels_to_custom_fields(
    labels: &[String],
    mappings: &FieldMappings,
) -> HashMap<String, String> {
    let mut custom_fields = HashMap::new();

    for label in labels {
        if let Some(mapping) = mappings.label_to_custom_field.get(label) {
            // Parse "field=value" format
            if let Some((field, value)) = mapping.split_once('=') {
                // Skip priority as it's handled separately
                if field != "priority" {
                    custom_fields.insert(field.to_string(), value.to_string());
                }
            }
        }
    }

    custom_fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_status() {
        let mut mappings = FieldMappings::default();
        mappings
            .status_mapping
            .insert("open".to_string(), "open".to_string());
        mappings
            .status_mapping
            .insert("closed".to_string(), "closed".to_string());

        assert_eq!(map_status("open", &mappings), "open");
        assert_eq!(map_status("closed", &mappings), "closed");
        assert_eq!(map_status("unknown", &mappings), "open"); // default
    }

    #[test]
    fn test_map_labels_to_custom_fields() {
        let mut mappings = FieldMappings::default();
        mappings
            .label_to_custom_field
            .insert("bug".to_string(), "type=bug".to_string());
        mappings
            .label_to_custom_field
            .insert("enhancement".to_string(), "type=feature".to_string());

        let labels = vec!["bug".to_string(), "other".to_string()];
        let custom_fields = map_labels_to_custom_fields(&labels, &mappings);

        assert_eq!(custom_fields.get("type"), Some(&"bug".to_string()));
    }
}
