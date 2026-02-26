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

    // Should have expected files
    assert!(files.contains_key("README.md"));
    assert!(files.contains_key("issues/README.md"));
    assert!(files.contains_key("templates/README.md"));
    assert!(files.contains_key("cspell.json"));
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
