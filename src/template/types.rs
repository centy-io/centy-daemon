use serde::Serialize;
use std::collections::HashMap;

/// Type of template (for determining folder path)
#[derive(Debug, Clone, Copy)]
pub enum TemplateType {
    Issue,
    Doc,
    Llm,
}

impl TemplateType {
    pub fn folder_name(&self) -> &'static str {
        match self {
            TemplateType::Issue => "issues",
            TemplateType::Doc => "docs",
            TemplateType::Llm => "llm",
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

/// Context for doc templates
/// Placeholders: {{title}}, {{content}}, {{slug}}, {{created_at}}, {{updated_at}}
#[derive(Debug, Clone, Serialize)]
pub struct DocTemplateContext {
    pub title: String,
    pub content: String,
    pub slug: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Context for LLM templates
/// Placeholders: {{issue_id}}, {{display_number}}, {{title}}, {{description}}, {{status}},
/// {{priority}}, {{priority_label}}, {{created_at}}, {{custom_fields}}, {{action}}, {{project_path}}
#[derive(Debug, Clone, Serialize)]
pub struct LlmTemplateContext {
    pub issue_id: String,
    pub display_number: u32,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: u32,
    pub priority_label: String,
    pub created_at: String,
    pub custom_fields: HashMap<String, String>,
    pub action: String,
    pub project_path: String,
}
