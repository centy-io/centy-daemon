use crate::logging::get_log_file_path;
use crate::server::error_mapping::ToStructuredError;
use serde::Serialize;
use std::fmt::Display;

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
#[path = "structured_error_tests.rs"]
mod tests;
