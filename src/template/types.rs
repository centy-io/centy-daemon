use serde::Serialize;
use std::collections::HashMap;

/// Type of template (for determining folder path)
#[derive(Debug, Clone, Copy)]
pub enum TemplateType {
    Issue,
}

impl TemplateType {
    #[must_use]
    pub fn folder_name(&self) -> &'static str {
        match self {
            TemplateType::Issue => "issues",
        }
    }
}

/// Context for issue templates
/// Placeholders: {{title}}, {{description}}, {{priority}}, {{priority_label}}, {{status}}, {{created_at}}, {{custom_fields}}
#[derive(Debug, Clone, Serialize)]
pub struct IssueTemplateContext {
    pub title: String,
    pub description: String,
    pub priority: u32,
    pub priority_label: String,
    pub status: String,
    pub created_at: String,
    pub custom_fields: HashMap<String, String>,
}

#[cfg(test)]
#[path = "types_tests_1.rs"]
mod tests_1;
