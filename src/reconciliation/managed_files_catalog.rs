use super::*;

#[test]
fn test_get_managed_files_returns_expected_files() {
    let files = get_managed_files();

    // Should have expected directories
    assert!(files.contains_key("issues/"));
    assert!(files.contains_key("docs/"));
    assert!(files.contains_key("archived/"));
    assert!(files.contains_key("assets/"));
    assert!(files.contains_key("templates/"));
    assert!(files.contains_key("templates/issues/"));
    assert!(files.contains_key("templates/docs/"));
    assert!(files.contains_key("comments/"));

    // Should have expected files
    assert!(files.contains_key("README.md"));
    assert!(files.contains_key("issues/README.md"));
    assert!(files.contains_key("templates/README.md"));
    assert!(files.contains_key("cspell.json"));
    assert!(files.contains_key("hooks.yaml"));
}

#[test]
fn test_get_managed_files_directories_have_correct_type() {
    let files = get_managed_files();

    // All directories should have Directory type
    let directories = [
        "issues/",
        "docs/",
        "archived/",
        "assets/",
        "templates/",
        "templates/issues/",
        "templates/docs/",
    ];
    for dir in directories {
        let template = files
            .get(dir)
            .unwrap_or_else(|| panic!("Should have {dir}"));
        assert_eq!(
            template.file_type,
            ManagedFileType::Directory,
            "Directory {dir} should have Directory type"
        );
        assert!(
            template.content.is_none(),
            "Directory {dir} should have no content"
        );
    }
}

#[test]
fn test_get_managed_files_files_have_correct_type() {
    let files = get_managed_files();

    // All files should have File type
    let regular_files = [
        "README.md",
        "issues/README.md",
        "templates/README.md",
        "cspell.json",
        "hooks.yaml",
    ];
    for file in regular_files {
        let template = files
            .get(file)
            .unwrap_or_else(|| panic!("Should have {file}"));
        assert_eq!(
            template.file_type,
            ManagedFileType::File,
            "File {file} should have File type"
        );
        assert!(
            template.content.is_some(),
            "File {file} should have content"
        );
    }
}

#[test]
fn test_managed_file_template_readme_content() {
    let files = get_managed_files();
    let readme = files.get("README.md").expect("Should have README.md");

    let content = readme.content.as_ref().expect("README should have content");
    assert!(content.contains("Centy Project"));
    assert!(content.contains("AI Assistant Instructions"));
    assert!(content.contains("centy create issue"));
}

#[test]
fn test_cspell_json_has_json_array_merge_strategy() {
    let files = get_managed_files();
    let cspell = files.get("cspell.json").expect("Should have cspell.json");
    assert_eq!(cspell.merge_strategy, Some(MergeStrategy::JsonArrayMerge));
    assert!(cspell.content.is_some());
    let content = cspell.content.as_ref().unwrap();
    assert!(content.contains("centy"));
}

#[test]
fn test_hooks_yaml_no_merge_strategy() {
    let files = get_managed_files();
    let hooks = files.get("hooks.yaml").expect("Should have hooks.yaml");
    assert!(hooks.merge_strategy.is_none());
    assert!(hooks.content.is_some());
}

#[test]
fn test_readme_no_merge_strategy() {
    let files = get_managed_files();
    let readme = &files["README.md"];
    assert!(readme.merge_strategy.is_none());
}

#[test]
fn test_merge_strategy_equality() {
    // Test PartialEq on MergeStrategy
    assert_eq!(MergeStrategy::JsonArrayMerge, MergeStrategy::JsonArrayMerge);
}

#[test]
fn test_comments_directory_has_no_content() {
    let files = get_managed_files();
    let comments = files.get("comments/").expect("Should have comments/");
    assert_eq!(comments.file_type, ManagedFileType::Directory);
    assert!(comments.content.is_none());
    assert!(comments.merge_strategy.is_none());
}

#[test]
fn test_managed_file_template_count() {
    let files = get_managed_files();
    // Should have at least 8 directories + 5 files = 13 entries
    assert!(files.len() >= 13);
}

#[test]
fn test_issues_readme_content() {
    let files = get_managed_files();
    let issues_readme = files
        .get("issues/README.md")
        .expect("Should have issues/README.md");
    let content = issues_readme.content.as_ref().expect("Should have content");
    assert!(!content.is_empty());
}

#[test]
fn test_templates_readme_content() {
    let files = get_managed_files();
    let templates_readme = files
        .get("templates/README.md")
        .expect("Should have templates/README.md");
    let content = templates_readme.content.as_ref().expect("Should have content");
    assert!(!content.is_empty());
}
