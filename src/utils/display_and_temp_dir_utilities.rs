use super::*;

#[test]
fn test_format_display_path_non_home() {
    // Paths outside home directory should remain unchanged
    let path = "/tmp/some/path";
    let result = format_display_path(path);
    assert_eq!(result, path);
}

#[test]
fn test_is_in_temp_dir_nonexistent_path_in_temp() {
    // A non-existent path inside the temp dir — canonicalize() will fail,
    // exercising the string-prefix fallback branch in is_in_temp_dir.
    let temp_dir = std::env::temp_dir();
    let nonexistent = temp_dir.join("centy-test-nonexistent-xyz-99999999");
    // Confirm the path doesn't actually exist (so canonicalize fails)
    assert!(!nonexistent.exists());
    assert!(is_in_temp_dir(&nonexistent));
}

#[test]
fn test_is_in_temp_dir_nonexistent_path_not_in_temp() {
    // A non-existent path NOT in temp dir — string fallback returns false.
    let nonexistent = std::path::Path::new("/nonexistent/project/path/xyz123");
    assert!(!nonexistent.exists());
    assert!(!is_in_temp_dir(nonexistent));
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
