use super::*;

#[test]
fn test_format_markdown_basic() {
    let input = "# Hello\n\nWorld";
    let output = format_markdown(input);
    assert!(output.ends_with('\n'));
    assert!(output.contains("# Hello"));
}

#[test]
fn test_format_markdown_trailing_newline() {
    let input = "# Test";
    let output = format_markdown(input);
    assert!(output.ends_with('\n'));
}

#[test]
fn test_format_markdown_already_has_newline() {
    let input = "# Test\n";
    let output = format_markdown(input);
    assert!(output.ends_with('\n'));
    // Should not have double newline
    assert!(!output.ends_with("\n\n"));
}

#[test]
fn test_format_markdown_converts_ellipsis() {
    // format_markdown converts ... to Unicode ellipsis …
    let input = "The users API...";
    let output = format_markdown(input);
    assert!(output.contains("…"), "Should convert ... to …");
}

#[test]
fn test_format_markdown_preserves_blockquote_content() {
    // Blockquotes may be reformatted but content should remain
    let input = "> **Planning Mode**: Some text\n\n# Title\n";
    let output = format_markdown(input);
    assert!(
        output.contains("> **Planning Mode**") || output.contains(" > **Planning Mode**"),
        "Should preserve blockquote content"
    );
}

#[test]
fn test_format_markdown_prepends_centy_header() {
    let input = "# Hello\n\nWorld";
    let output = format_markdown(input);
    assert!(
        output.starts_with(CENTY_HEADER_MD),
        "Should prepend the managed-by header"
    );
}

#[test]
fn test_format_markdown_header_idempotent() {
    let input = "# Hello\n\nWorld";
    let once = format_markdown(input);
    let twice = format_markdown(&once);
    assert_eq!(
        once.matches(CENTY_HEADER_MD).count(),
        1,
        "Header should appear exactly once after first call"
    );
    assert_eq!(
        twice.matches(CENTY_HEADER_MD).count(),
        1,
        "Header should not be duplicated after second call"
    );
}

#[test]
fn test_with_yaml_header_prepends() {
    let content = "key: value\n";
    let result = with_yaml_header(content);
    assert!(result.starts_with(CENTY_HEADER_YAML));
    assert!(result.contains("key: value"));
}

#[test]
fn test_with_yaml_header_idempotent() {
    let content = "key: value\n";
    let once = with_yaml_header(content);
    let twice = with_yaml_header(&once);
    assert_eq!(
        once.matches(CENTY_HEADER_YAML).count(),
        1,
        "YAML header should appear exactly once after first call"
    );
    assert_eq!(
        twice.matches(CENTY_HEADER_YAML).count(),
        1,
        "YAML header should not be duplicated after second call"
    );
}

#[test]
fn test_strip_centy_md_header_removes_header() {
    let with_header = format!("{CENTY_HEADER_MD}\n---\nkey: value\n---\n# Title\n");
    let stripped = strip_centy_md_header(&with_header);
    assert!(!stripped.starts_with(CENTY_HEADER_MD));
    assert!(stripped.starts_with("---"));
}

#[test]
fn test_strip_centy_md_header_no_op_without_header() {
    let content = "---\nkey: value\n---\n# Title\n";
    let stripped = strip_centy_md_header(content);
    assert_eq!(stripped, content);
}
