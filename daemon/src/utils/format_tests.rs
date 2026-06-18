use super::*;

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
