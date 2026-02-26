use super::super::crud::{Issue, IssueMetadataFlat};
use super::super::metadata::IssueMetadata;
use super::super::org_registry::get_next_org_display_number;
use super::super::priority::{default_priority, validate_priority};
use super::types::{CreateIssueOptions, IssueError};
use crate::registry::get_project_info;
use std::collections::HashMap;
use std::path::Path;

pub async fn resolve_org_info(
    project_path: &Path,
    is_org_issue: bool,
) -> Result<(Option<String>, Option<u32>), IssueError> {
    if !is_org_issue {
        return Ok((None, None));
    }

    let project_path_str = project_path.to_string_lossy().to_string();
    let project_info = get_project_info(&project_path_str)
        .await
        .map_err(|e| IssueError::RegistryError(e.to_string()))?;

    match project_info.and_then(|p| p.organization_slug) {
        Some(slug) => {
            let org_num = get_next_org_display_number(&slug).await?;
            Ok((Some(slug), Some(org_num)))
        }
        None => Err(IssueError::NoOrganization),
    }
}

pub fn resolve_priority(
    priority_opt: Option<u32>,
    config: Option<&crate::config::CentyConfig>,
    priority_levels: u32,
) -> Result<u32, IssueError> {
    match priority_opt {
        Some(p) => {
            validate_priority(p, priority_levels)?;
            Ok(p)
        }
        None => Ok(config
            .and_then(|c| c.defaults.get("priority"))
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or_else(|| default_priority(priority_levels))),
    }
}

pub fn build_custom_fields(
    config: Option<&crate::config::CentyConfig>,
    provided_fields: &HashMap<String, String>,
) -> HashMap<String, serde_json::Value> {
    let mut fields: HashMap<String, serde_json::Value> = HashMap::new();

    if let Some(config) = config {
        for field in &config.custom_fields {
            if let Some(default_value) = &field.default_value {
                fields.insert(
                    field.name.clone(),
                    serde_json::Value::String(default_value.clone()),
                );
            }
        }
    }

    for (key, value) in provided_fields {
        fields.insert(key.clone(), serde_json::Value::String(value.clone()));
    }

    fields
}

pub fn build_issue_for_sync(
    issue_id: &str,
    options: &CreateIssueOptions,
    display_number: u32,
    metadata: &IssueMetadata,
) -> Issue {
    #[allow(deprecated)]
    Issue {
        id: issue_id.to_string(),
        issue_number: issue_id.to_string(),
        title: options.title.clone(),
        description: options.description.clone(),
        metadata: IssueMetadataFlat {
            display_number,
            status: metadata.common.status.clone(),
            priority: metadata.common.priority,
            created_at: metadata.common.created_at.clone(),
            updated_at: metadata.common.updated_at.clone(),
            custom_fields: options.custom_fields.clone(),
            draft: metadata.draft,
            deleted_at: metadata.deleted_at.clone(),
            is_org_issue: metadata.is_org_issue,
            org_slug: metadata.org_slug.clone(),
            org_display_number: metadata.org_display_number,
        },
    }
}

