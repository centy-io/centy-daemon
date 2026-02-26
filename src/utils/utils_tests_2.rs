use super::*;

#[test]
fn test_format_display_path_non_home() {
    // Paths outside home directory should remain unchanged
    let path = "/tmp/some/path";
    let result = format_display_path(path);
    assert_eq!(result, path);
}

#[test]
fn test_format_display_path_home() {
    // Get the actual home directory for this system
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        let test_path = format!("{home_str}/projects/myapp");
        let result = format_display_path(&test_path);
        assert_eq!(result, "~/projects/myapp");
    }
}

#[test]
fn test_is_in_temp_dir_temp_path() {
    let temp_dir = std::env::temp_dir();
    let test_path = temp_dir.join("some-project");
    assert!(is_in_temp_dir(&test_path));
}

#[test]
fn test_is_in_temp_dir_non_temp_path() {
    let home_dir =
        dirs::home_dir().unwrap_or_else(|| std::path::Path::new("/home/user").to_path_buf());
    let test_path = home_dir.join("projects/my-project");
    assert!(!is_in_temp_dir(&test_path));
}

#[test]
fn test_is_in_temp_dir_nested_temp_path() {
    let temp_dir = std::env::temp_dir();
    let test_path = temp_dir.join("centy-abc123-20251215").join("subdir");
    assert!(is_in_temp_dir(&test_path));
}
