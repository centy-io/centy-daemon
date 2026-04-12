//! Tests for `execute/file_ops.rs` covering missed branches.
#![allow(clippy::unwrap_used, clippy::expect_used)]

// `super` here = the `execute` module (execute/mod.rs)
// `super::super` = `reconciliation` module
use super::super::managed_files::{get_managed_files, ManagedFileTemplate, MergeStrategy};
use super::file_ops::{create_file, merge_file};
use crate::manifest::ManagedFileType;
use std::collections::HashMap;
use tokio::fs;

// ─── create_file ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_file_template_not_found_error() {
    let temp = tempfile::tempdir().unwrap();
    let templates: HashMap<String, ManagedFileTemplate> = HashMap::new();

    let result = create_file(temp.path(), "nonexistent/path.md", &templates).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_file_directory_type() {
    let temp = tempfile::tempdir().unwrap();
    let mut templates = HashMap::new();
    templates.insert(
        "issues/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );

    create_file(temp.path(), "issues/", &templates)
        .await
        .unwrap();
    assert!(temp.path().join("issues").exists());
    assert!(temp.path().join("issues").is_dir());
}

#[tokio::test]
async fn test_create_file_file_type_with_content() {
    let temp = tempfile::tempdir().unwrap();
    let mut templates = HashMap::new();
    templates.insert(
        "README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some("# Hello World\n".to_string()),
            merge_strategy: None,
        },
    );

    create_file(temp.path(), "README.md", &templates)
        .await
        .unwrap();
    let content = fs::read_to_string(temp.path().join("README.md"))
        .await
        .unwrap();
    assert_eq!(content, "# Hello World\n");
}

#[tokio::test]
async fn test_create_file_file_type_no_content() {
    let temp = tempfile::tempdir().unwrap();
    let mut templates = HashMap::new();
    templates.insert(
        "empty.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: None,
            merge_strategy: None,
        },
    );

    create_file(temp.path(), "empty.md", &templates)
        .await
        .unwrap();
    let content = fs::read_to_string(temp.path().join("empty.md"))
        .await
        .unwrap();
    assert_eq!(content, "");
}

#[tokio::test]
async fn test_create_file_creates_parent_dirs() {
    let temp = tempfile::tempdir().unwrap();
    let mut templates = HashMap::new();
    templates.insert(
        "nested/deep/file.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some("content".to_string()),
            merge_strategy: None,
        },
    );

    create_file(temp.path(), "nested/deep/file.md", &templates)
        .await
        .unwrap();
    assert!(temp
        .path()
        .join("nested")
        .join("deep")
        .join("file.md")
        .exists());
}

#[tokio::test]
async fn test_create_file_trims_trailing_slash() {
    let temp = tempfile::tempdir().unwrap();
    let mut templates = HashMap::new();
    templates.insert(
        "test_dir/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );

    create_file(temp.path(), "test_dir/", &templates)
        .await
        .unwrap();
    assert!(temp.path().join("test_dir").exists());
    assert!(temp.path().join("test_dir").is_dir());
}

// ─── merge_file ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_merge_file_template_not_found_error() {
    let temp = tempfile::tempdir().unwrap();
    let templates: HashMap<String, ManagedFileTemplate> = HashMap::new();

    let result = merge_file(temp.path(), "nonexistent.json", &templates).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_merge_file_none_strategy_overwrites() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("config.yaml");
    fs::write(&file_path, b"old: content").await.unwrap();

    let mut templates = HashMap::new();
    templates.insert(
        "config.yaml".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some("new: content\n".to_string()),
            merge_strategy: None,
        },
    );

    merge_file(temp.path(), "config.yaml", &templates)
        .await
        .unwrap();
    let content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(content, "new: content\n");
}

#[tokio::test]
async fn test_merge_file_json_array_merge_strategy() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("cspell.json");
    let existing = r#"{"version":"0.1","language":"en","words":["alpha"],"ignorePaths":[]}"#;
    fs::write(&file_path, existing.as_bytes()).await.unwrap();

    let mut templates = HashMap::new();
    templates.insert(
        "cspell.json".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(
                r#"{"version":"0.2","language":"en","words":["beta"],"ignorePaths":[]}"#
                    .to_string(),
            ),
            merge_strategy: Some(MergeStrategy::JsonArrayMerge),
        },
    );

    merge_file(temp.path(), "cspell.json", &templates)
        .await
        .unwrap();
    let content = fs::read_to_string(&file_path).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Version should be updated to template's version
    assert_eq!(parsed["version"], "0.2");

    // Words should be merged (union)
    let words: Vec<&str> = parsed["words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(words.contains(&"alpha"));
    assert!(words.contains(&"beta"));
}

#[tokio::test]
async fn test_merge_file_json_invalid_existing_content() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("cspell.json");
    fs::write(&file_path, b"invalid json {{{").await.unwrap();

    let mut templates = HashMap::new();
    templates.insert(
        "cspell.json".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(r#"{"version":"0.2","words":[]}"#.to_string()),
            merge_strategy: Some(MergeStrategy::JsonArrayMerge),
        },
    );

    let result = merge_file(temp.path(), "cspell.json", &templates).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_file_using_managed_templates() {
    let temp = tempfile::tempdir().unwrap();
    let templates = get_managed_files();

    create_file(temp.path(), "issues/", &templates)
        .await
        .unwrap();
    assert!(temp.path().join("issues").exists());

    create_file(temp.path(), "README.md", &templates)
        .await
        .unwrap();
    let content = fs::read_to_string(temp.path().join("README.md"))
        .await
        .unwrap();
    assert!(!content.is_empty());
}
