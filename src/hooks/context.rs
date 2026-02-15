use serde::Serialize;
use std::collections::HashMap;

use super::config::{HookOperation, Phase};

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
mod tests {
    use super::*;

    #[test]
    fn test_context_env_vars() {
        let ctx = HookContext::new(
            Phase::Pre,
            "issue",
            HookOperation::Create,
            "/tmp/project",
            Some("issue-123"),
            None,
            None,
        );

        let vars = ctx.to_env_vars();
        assert_eq!(vars.get("CENTY_PHASE").unwrap(), "pre");
        assert_eq!(vars.get("CENTY_ITEM_TYPE").unwrap(), "issue");
        assert_eq!(vars.get("CENTY_OPERATION").unwrap(), "create");
        assert_eq!(vars.get("CENTY_PROJECT_PATH").unwrap(), "/tmp/project");
        assert_eq!(vars.get("CENTY_ITEM_ID").unwrap(), "issue-123");
    }

    #[test]
    fn test_context_env_vars_no_item_id() {
        let ctx = HookContext::new(
            Phase::Pre,
            "doc",
            HookOperation::Create,
            "/tmp/project",
            None,
            None,
            None,
        );

        let vars = ctx.to_env_vars();
        assert!(!vars.contains_key("CENTY_ITEM_ID"));
    }

    #[test]
    fn test_context_to_json() {
        let ctx = HookContext::new(
            Phase::Post,
            "issue",
            HookOperation::Create,
            "/tmp/project",
            Some("issue-123"),
            Some(serde_json::json!({"title": "Test"})),
            Some(true),
        );

        let json = ctx.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["phase"], "post");
        assert_eq!(parsed["item_type"], "issue");
        assert_eq!(parsed["operation"], "create");
        assert_eq!(parsed["project_path"], "/tmp/project");
        assert_eq!(parsed["item_id"], "issue-123");
        assert_eq!(parsed["request_data"]["title"], "Test");
        assert_eq!(parsed["success"], true);
    }
}
