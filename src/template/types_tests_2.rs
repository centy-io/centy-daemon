use super::*;

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
