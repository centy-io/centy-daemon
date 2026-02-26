use super::*;

#[test]
fn test_template_type_folder_name() {
    assert_eq!(TemplateType::Issue.folder_name(), "issues");
    assert_eq!(TemplateType::Doc.folder_name(), "docs");
}

#[test]
fn test_template_engine_creation() {
    let engine = TemplateEngine::new();
    // Basic test that engine can be created
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

    let docs_path = TemplateEngine::get_template_type_path(project_path, TemplateType::Doc);
    assert_eq!(docs_path, Path::new("/test/project/.centy/templates/docs"));
}
