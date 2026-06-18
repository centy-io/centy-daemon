use super::*;
use std::collections::HashMap;

#[test]
fn test_template_type_folder_name() {
    assert_eq!(TemplateType::Issue.folder_name(), "issues");
}

#[test]
fn test_template_engine_creation() {
    let engine = TemplateEngine::new();
    // Basic test that engine can be created
    assert!(engine.handlebars.get_templates().is_empty());
}

#[test]
fn test_template_engine_default() {
    let engine = TemplateEngine::default();
    assert!(engine.handlebars.get_templates().is_empty());
}

#[test]
fn test_get_templates_path() {
    let project_path = Path::new("/test/project");
    let templates_path = TemplateEngine::get_templates_path(project_path);
    assert_eq!(templates_path, Path::new("/test/project/.centy/templates"));
}

#[test]
fn test_get_template_type_path() {
    let project_path = Path::new("/test/project");

    let issues_path = TemplateEngine::get_template_type_path(project_path, TemplateType::Issue);
    assert_eq!(
        issues_path,
        Path::new("/test/project/.centy/templates/issues")
    );
}

#[tokio::test]
async fn test_load_template_not_found_returns_error() {
    let engine = TemplateEngine::new();
    let project_path = Path::new("/nonexistent/project/path");
    let result = engine
        .load_template(project_path, TemplateType::Issue, "nonexistent-template")
        .await;
    match result {
        Err(TemplateError::TemplateNotFound(name)) => {
            assert_eq!(name, "nonexistent-template.md");
        }
        other => panic!("Expected TemplateNotFound, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_render_issue_with_real_template() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project_path = dir.path();

    // Create the template directory structure
    let template_dir = project_path.join(".centy/templates/issues");
    tokio::fs::create_dir_all(&template_dir)
        .await
        .expect("create template dir");

    // Write a simple template file
    tokio::fs::write(
        template_dir.join("default.md"),
        "# {{title}}\n\n{{description}}",
    )
    .await
    .expect("write template");

    let engine = TemplateEngine::new();
    let context = IssueTemplateContext {
        title: "Test Issue".to_string(),
        description: "A test description".to_string(),
        priority: 2,
        priority_label: "Medium".to_string(),
        status: "open".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        custom_fields: HashMap::new(),
    };

    let result = engine
        .render_issue(project_path, "default", &context)
        .await
        .expect("Should render template");

    assert!(result.contains("Test Issue"));
    assert!(result.contains("A test description"));
}
