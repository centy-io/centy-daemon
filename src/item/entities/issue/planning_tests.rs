use super::*;

#[test]
fn test_is_planning_status() {
    assert!(is_planning_status("planning"));
    assert!(!is_planning_status("open"));
    assert!(!is_planning_status("in-progress"));
    assert!(!is_planning_status("closed"));
    assert!(!is_planning_status("Planning")); // case-sensitive
}

#[test]
fn test_has_planning_note_at_start() {
    let content = format!("{PLANNING_NOTE}# Title\n\nDescription");
    assert!(has_planning_note(&content));
}

#[test]
fn test_has_planning_note_false() {
    let content = "# Title\n\nDescription";
    assert!(!has_planning_note(content));
}

#[test]
fn test_add_planning_note() {
    let content = "# Title\n\nDescription\n";
    let result = add_planning_note(content);
    assert!(result.starts_with(PLANNING_NOTE));
    assert!(result.contains("# Title"));
}

#[test]
fn test_add_planning_note_idempotent() {
    let content = format!("{PLANNING_NOTE}# Title\n");
    let result = add_planning_note(&content);
    // Should not add duplicate note
    assert_eq!(result.matches("> **Planning Mode**").count(), 1);
}

#[test]
fn test_remove_planning_note() {
    let content = format!("{PLANNING_NOTE}# Title\n\nDescription\n");
    let result = remove_planning_note(&content);
    assert!(!result.contains("Planning Mode"));
    assert!(result.starts_with("# Title"));
}

#[test]
fn test_remove_planning_note_when_absent() {
    let content = "# Title\n\nDescription\n";
    let result = remove_planning_note(content);
    assert_eq!(result, content);
}

#[test]
fn test_remove_manually_edited_note() {
    // User might have slightly modified the note
    let content = "> **Planning Mode**: Custom text here\n\n# Title\n";
    let result = remove_planning_note(content);
    assert!(!result.contains("Planning Mode"));
    assert!(result.contains("# Title"));
}

#[test]
fn test_remove_formatted_planning_note() {
    // When formatted by markdown formatter, the note may have leading spaces
    // and be wrapped differently
    let content = " > \n > **Planning Mode**: Do not implement code changes.\n\nSome description";
    let result = remove_planning_note(content);
    assert!(
        !result.contains("Planning Mode"),
        "Should not contain Planning Mode, got: {result:?}"
    );
    assert!(
        !result.contains("> "),
        "Should not contain blockquote marker, got: {result:?}"
    );
    assert_eq!(result.trim(), "Some description");
}
