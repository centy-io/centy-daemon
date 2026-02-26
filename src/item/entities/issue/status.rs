use crate::config::item_type_config::read_item_type_config;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatusError {
    #[error("Invalid status '{0}'. Allowed values: {1}")]
    InvalidStatus(String, String),
}

/// Validate that a status is in the allowed states list.
/// Returns Ok(()) if valid, or Err with allowed states for helpful error message.
pub fn validate_status(status: &str, allowed_states: &[String]) -> Result<(), StatusError> {
    if allowed_states.iter().any(|s| s == status) {
        Ok(())
    } else {
        let allowed_list = allowed_states.join(", ");
        Err(StatusError::InvalidStatus(status.to_string(), allowed_list))
    }
}

/// Validate a status against the item-type config for the given project.
/// Reads `.centy/<item_type>/config.yaml` and checks the `statuses` field.
/// If no config or no statuses are defined, validation passes (permissive).
pub async fn validate_status_for_project(
    project_path: &Path,
    item_type: &str,
    status: &str,
) -> Result<(), StatusError> {
    let itc = read_item_type_config(project_path, item_type)
        .await
        .ok()
        .flatten();
    if let Some(ref config) = itc {
        if !config.statuses.is_empty() {
            return validate_status(status, &config.statuses);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "status_tests.rs"]
mod tests;
