use super::config::{HookOperation, Phase};
use serde::Serialize;
use std::collections::HashMap;

/// Context passed to hook scripts via env vars and stdin JSON
#[derive(Debug, Clone, Serialize)]
pub struct HookContext {
    pub phase: String,
    pub item_type: String,
    pub operation: String,
    pub project_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
}

impl HookContext {
    pub fn new(
        phase: Phase,
        item_type: &str,
        operation: HookOperation,
        project_path: &str,
        item_id: Option<&str>,
        request_data: Option<serde_json::Value>,
        success: Option<bool>,
    ) -> Self {
        Self {
            phase: phase.as_str().to_string(),
            item_type: item_type.to_string(),
            operation: operation.as_str().to_string(),
            project_path: project_path.to_string(),
            item_id: item_id.map(String::from),
            request_data,
            success,
        }
    }

    /// Convert to environment variables for the hook process
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("CENTY_PHASE".to_string(), self.phase.clone());
        vars.insert("CENTY_ITEM_TYPE".to_string(), self.item_type.clone());
        vars.insert("CENTY_OPERATION".to_string(), self.operation.clone());
        vars.insert("CENTY_PROJECT_PATH".to_string(), self.project_path.clone());
        if let Some(ref id) = self.item_id {
            vars.insert("CENTY_ITEM_ID".to_string(), id.clone());
        }
        vars
    }

    /// Convert to JSON string for stdin piping
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
