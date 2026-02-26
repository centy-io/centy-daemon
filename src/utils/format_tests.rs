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
