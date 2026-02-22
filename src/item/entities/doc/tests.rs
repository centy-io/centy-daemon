use super::content::{generate_doc_content, parse_doc_content};
use super::helpers::{escape_yaml_string, slugify, validate_slug};
use super::types::DocMetadata;

#[test]
fn test_slugify() {
    assert_eq!(slugify("Hello World"), "hello-world");
    assert_eq!(slugify("Getting Started Guide"), "getting-started-guide");
    assert_eq!(slugify("API v2"), "api-v2");
    assert_eq!(slugify("  Spaces  "), "spaces");
    assert_eq!(slugify("multiple---hyphens"), "multiple-hyphens");
    assert_eq!(slugify("Under_score"), "under-score");
}

#[test]
fn test_validate_slug() {
    assert!(validate_slug("hello-world").is_ok());
    assert!(validate_slug("api-v2").is_ok());
    assert!(validate_slug("").is_err());
    assert!(validate_slug("-start").is_err());
    assert!(validate_slug("end-").is_err());
    assert!(validate_slug("has space").is_err());
}

#[test]
fn test_parse_doc_content_with_frontmatter() {
    let content = r#"---
title: "My Doc"
createdAt: "2024-01-01T00:00:00Z"
updatedAt: "2024-01-02T00:00:00Z"
---

# My Doc

This is the content."#;

    let (title, body, metadata) = parse_doc_content(content);
    assert_eq!(title, "My Doc");
    assert_eq!(body, "This is the content.");
    assert_eq!(metadata.created_at, "2024-01-01T00:00:00Z");
    assert_eq!(metadata.updated_at, "2024-01-02T00:00:00Z");
}

#[test]
fn test_parse_doc_content_without_frontmatter() {
    let content = "# Simple Doc\n\nJust some content here.";
    let (title, body, _metadata) = parse_doc_content(content);
    assert_eq!(title, "Simple Doc");
    assert_eq!(body, "Just some content here.");
}

#[test]
fn test_generate_doc_content() {
    let metadata = DocMetadata {
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
        deleted_at: None,
        is_org_doc: false,
        org_slug: None,
    };
    let content = generate_doc_content("Test Title", "Body text", &metadata);

    assert!(content.contains("title: \"Test Title\""));
    assert!(content.contains("# Test Title"));
    assert!(content.contains("Body text"));
}

#[test]
fn test_escape_yaml_string() {
    assert_eq!(escape_yaml_string("simple"), "simple");
    assert_eq!(escape_yaml_string("with \"quotes\""), "with \\\"quotes\\\"");
    assert_eq!(escape_yaml_string("back\\slash"), "back\\\\slash");
}
