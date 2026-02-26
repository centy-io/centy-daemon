use super::*;

#[test]
fn test_parse_issue_md_with_description() {
    let content = "# My Issue Title\n\nThis is the description.\nWith multiple lines.";
    let (title, description) = parse_issue_md(content);
    assert_eq!(title, "My Issue Title");
    assert_eq!(
        description,
        "This is the description.\nWith multiple lines."
    );
}

#[test]
fn test_parse_issue_md_title_only() {
    let content = "# My Issue Title\n";
    let (title, description) = parse_issue_md(content);
    assert_eq!(title, "My Issue Title");
    assert_eq!(description, "");
}

#[test]
fn test_parse_issue_md_empty() {
    let content = "";
    let (title, description) = parse_issue_md(content);
    assert_eq!(title, "");
    assert_eq!(description, "");
}
