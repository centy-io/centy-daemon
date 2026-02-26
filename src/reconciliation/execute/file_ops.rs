use super::super::managed_files::{merge_json_content, ManagedFileTemplate, MergeStrategy};
use super::types::ExecuteError;
use crate::manifest::ManagedFileType;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
/// Create a file or directory from template
pub async fn create_file(
    centy_path: &Path,
    relative_path: &str,
    templates: &HashMap<String, ManagedFileTemplate>,
) -> Result<(), ExecuteError> {
    let template = templates
        .get(relative_path)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Template not found"))?;
    let full_path = centy_path.join(relative_path.trim_end_matches('/'));
    match &template.file_type {
        ManagedFileType::Directory => {
            fs::create_dir_all(&full_path).await?;
        }
        ManagedFileType::File => {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            let content = template.content.as_deref().unwrap_or("");
            fs::write(&full_path, content).await?;
        }
    }
    Ok(())
}
/// Merge a file on disk with its template content using the template's merge strategy
pub async fn merge_file(
    centy_path: &Path,
    relative_path: &str,
    templates: &HashMap<String, ManagedFileTemplate>,
) -> Result<(), ExecuteError> {
    let template = templates
        .get(relative_path)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Template not found"))?;
    let full_path = centy_path.join(relative_path.trim_end_matches('/'));
    let template_content = template.content.as_deref().unwrap_or("");
    match &template.merge_strategy {
        Some(MergeStrategy::JsonArrayMerge) => {
            let existing_content = fs::read_to_string(&full_path).await?;
            let merged = merge_json_content(&existing_content, template_content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            fs::write(&full_path, merged).await?;
        }
        None => {
            fs::write(&full_path, template_content).await?;
        }
    }
    Ok(())
}
