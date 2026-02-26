use super::*;

#[test]
fn test_get_managed_files_count() {
    let files = get_managed_files();

    // 7 directories + 4 files = 11 total
    assert_eq!(files.len(), 11);
}

#[test]
fn test_managed_file_template_struct() {
    let template = ManagedFileTemplate {
        file_type: ManagedFileType::File,
        content: Some("test content".to_string()),
        merge_strategy: None,
    };

    assert_eq!(template.file_type, ManagedFileType::File);
    assert_eq!(template.content, Some("test content".to_string()));
    assert!(template.merge_strategy.is_none());
}

#[test]
fn test_managed_file_template_clone() {
    let template = ManagedFileTemplate {
        file_type: ManagedFileType::Directory,
        content: None,
        merge_strategy: None,
    };

    let cloned = template.clone();
    assert_eq!(cloned.file_type, ManagedFileType::Directory);
    assert!(cloned.content.is_none());
}

#[test]
fn test_managed_file_template_debug() {
    let template = ManagedFileTemplate {
        file_type: ManagedFileType::File,
        content: Some("test".to_string()),
        merge_strategy: None,
    };

    let debug_str = format!("{template:?}");
    assert!(debug_str.contains("ManagedFileTemplate"));
    assert!(debug_str.contains("File"));
    assert!(debug_str.contains("test"));
}

#[test]
fn test_cspell_has_merge_strategy() {
    let files = get_managed_files();
    let cspell = files.get("cspell.json").expect("Should have cspell.json");
    assert_eq!(
        cspell.merge_strategy,
        Some(MergeStrategy::JsonArrayMerge),
        "cspell.json should have JsonArrayMerge strategy"
    );
}

#[test]
fn test_non_cspell_files_have_no_merge_strategy() {
    let files = get_managed_files();
    for (path, template) in &files {
        if path != "cspell.json" {
            assert!(
                template.merge_strategy.is_none(),
                "{path} should not have a merge strategy"
            );
        }
    }
}
