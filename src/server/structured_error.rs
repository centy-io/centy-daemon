use std::fmt::Display;

use serde::Serialize;

use crate::logging::get_log_file_path;
use crate::server::error_mapping::ToStructuredError;

#[derive(Serialize)]
pub struct ErrorMessage {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tip: Option<String>,
    pub code: String,
}

#[derive(Serialize)]
pub struct StructuredError {
    pub cwd: String,
    pub logs: String,
    pub messages: Vec<ErrorMessage>,
}

impl StructuredError {
    pub fn new(cwd: &str, code: &str, message: String) -> Self {
        Self {
            cwd: cwd.to_string(),
            logs: get_log_file_path().to_string(),
            messages: vec![ErrorMessage {
                message,
                tip: None,
                code: code.to_string(),
            }],
        }
    }

    #[must_use]
    pub fn with_tip(mut self, tip: &str) -> Self {
        if let Some(msg) = self.messages.first_mut() {
            msg.tip = Some(tip.to_string());
        }
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            // Fallback: produce a minimal valid JSON manually
            r#"{"cwd":"","logs":"","messages":[{"message":"serialization error","code":"INTERNAL_ERROR"}]}"#.to_string()
        })
    }
}

/// Convenience function to convert a domain error into a structured JSON error string.
pub fn to_error_json<E: ToStructuredError + Display>(cwd: &str, err: &E) -> String {
    let (code, tip) = err.error_code_and_tip();
    let mut se = StructuredError::new(cwd, code, err.to_string());
    if let Some(tip) = tip {
        se = se.with_tip(tip);
    }
    se.to_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_error_json_format() {
        let se = StructuredError::new(
            "/tmp/project",
            "ITEM_NOT_FOUND",
            "Issue not found: abc".to_string(),
        );
        let json = se.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["cwd"], "/tmp/project");
        assert_eq!(parsed["messages"][0]["code"], "ITEM_NOT_FOUND");
        assert_eq!(parsed["messages"][0]["message"], "Issue not found: abc");
        assert!(parsed["messages"][0].get("tip").is_none());
    }

    #[test]
    fn test_structured_error_with_tip() {
        let se = StructuredError::new(
            "/tmp/project",
            "NOT_INITIALIZED",
            "Project not initialized".to_string(),
        )
        .with_tip("Run 'centy init' to initialize the project");
        let json = se.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed["messages"][0]["tip"],
            "Run 'centy init' to initialize the project"
        );
    }

    #[test]
    fn test_structured_error_tip_skipped_when_none() {
        let se = StructuredError::new("/tmp/project", "IO_ERROR", "file not found".to_string());
        let json = se.to_json();
        assert!(!json.contains("\"tip\""));
    }

    #[test]
    fn test_structured_error_logs_field() {
        let se = StructuredError::new("", "TEST", "test".to_string());
        let json = se.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        // logs field should exist (may be empty in test context since OnceLock not set)
        assert!(parsed.get("logs").is_some());
    }
}
