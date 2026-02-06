use serde::Serialize;
use std::collections::HashMap;

/// Type of template (for determining folder path)
#[derive(Debug, Clone, Copy)]
pub enum TemplateType {
    Issue,
    Doc,
}

impl TemplateType {
    #[must_use]
    pub fn folder_name(&self) -> &'static str {
        match self {
            TemplateType::Issue => "issues",
            TemplateType::Doc => "docs",
        }
    }
}

/// Context for issue templates
/// Placeholders: {{title}}, {{description}}, {{priority}}, {{`priority_label`}}, {{status}}, {{`created_at`}}, {{`custom_fields`}}
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
/// Placeholders: {{title}}, {{content}}, {{slug}}, {{`created_at`}}, {{`updated_at`}}
#[derive(Debug, Clone, Serialize)]
pub struct DocTemplateContext {
    pub title: String,
    pub content: String,
    pub slug: String,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_type_issue_folder_name() {
        assert_eq!(TemplateType::Issue.folder_name(), "issues");
    }

    #[test]
    fn test_template_type_doc_folder_name() {
        assert_eq!(TemplateType::Doc.folder_name(), "docs");
    }

    #[test]
    fn test_template_type_debug() {
        let debug = format!("{:?}", TemplateType::Issue);
        assert!(debug.contains("Issue"));
        let debug = format!("{:?}", TemplateType::Doc);
        assert!(debug.contains("Doc"));
    }

    #[test]
    fn test_template_type_clone() {
        let t = TemplateType::Issue;
        let cloned = t;
        assert_eq!(cloned.folder_name(), "issues");
    }

    #[test]
    fn test_issue_template_context_serialization() {
        let ctx = IssueTemplateContext {
            title: "Bug Report".to_string(),
            description: "Something broke".to_string(),
            priority: 1,
            priority_label: "high".to_string(),
            status: "open".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            custom_fields: HashMap::new(),
        };

        let json = serde_json::to_string(&ctx).expect("Should serialize");
        assert!(json.contains("\"title\":\"Bug Report\""));
        assert!(json.contains("\"priority\":1"));
        assert!(json.contains("\"priority_label\":\"high\""));
        assert!(json.contains("\"status\":\"open\""));
    }

    #[test]
    fn test_issue_template_context_with_custom_fields() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert("environment".to_string(), "production".to_string());
        custom_fields.insert("browser".to_string(), "firefox".to_string());

        let ctx = IssueTemplateContext {
            title: "Bug".to_string(),
            description: "desc".to_string(),
            priority: 2,
            priority_label: "medium".to_string(),
            status: "open".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            custom_fields,
        };

        let json = serde_json::to_string(&ctx).expect("Should serialize");
        assert!(json.contains("environment"));
        assert!(json.contains("production"));
    }

    #[test]
    fn test_issue_template_context_clone() {
        let ctx = IssueTemplateContext {
            title: "Test".to_string(),
            description: "desc".to_string(),
            priority: 1,
            priority_label: "high".to_string(),
            status: "open".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            custom_fields: HashMap::new(),
        };

        let cloned = ctx.clone();
        assert_eq!(cloned.title, "Test");
        assert_eq!(cloned.priority, 1);
    }

    #[test]
    fn test_doc_template_context_serialization() {
        let ctx = DocTemplateContext {
            title: "API Docs".to_string(),
            content: "# API\nEndpoints here".to_string(),
            slug: "api-docs".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-06-15T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&ctx).expect("Should serialize");
        assert!(json.contains("\"title\":\"API Docs\""));
        assert!(json.contains("\"slug\":\"api-docs\""));
        assert!(json.contains("\"content\""));
    }

    #[test]
    fn test_doc_template_context_clone() {
        let ctx = DocTemplateContext {
            title: "Test".to_string(),
            content: "content".to_string(),
            slug: "test".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let cloned = ctx.clone();
        assert_eq!(cloned.slug, "test");
    }

    #[test]
    fn test_doc_template_context_debug() {
        let ctx = DocTemplateContext {
            title: "Test".to_string(),
            content: "content".to_string(),
            slug: "test".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let debug = format!("{ctx:?}");
        assert!(debug.contains("DocTemplateContext"));
        assert!(debug.contains("test"));
    }
}
