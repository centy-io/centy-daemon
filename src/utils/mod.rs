mod format;
mod hash;

pub use format::format_markdown;
pub use hash::{compute_file_hash, compute_hash};

use std::path::Path;

/// The name of the centy folder
pub const CENTY_FOLDER: &str = ".centy";

/// The name of the manifest file
pub const MANIFEST_FILE: &str = ".centy-manifest.json";

/// Current centy version
pub const CENTY_VERSION: &str = "0.1.0";

/// Get the path to the .centy folder
#[must_use]
pub fn get_centy_path(project_path: &Path) -> std::path::PathBuf {
    project_path.join(CENTY_FOLDER)
}

/// Get the path to the manifest file
#[must_use]
pub fn get_manifest_path(project_path: &Path) -> std::path::PathBuf {
    get_centy_path(project_path).join(MANIFEST_FILE)
}

/// Get current timestamp in ISO 8601 format
#[must_use]
pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Format a path for display, replacing home directory with ~/
#[must_use]
pub fn format_display_path(path: &str) -> String {
    replace_homedir::replace_homedir(path, "~")
}

/// Check if a path is inside the system's temporary directory.
/// Uses `std::env::temp_dir()` to detect temp paths cross-platform.
#[must_use]
pub fn is_in_temp_dir(path: &Path) -> bool {
    let temp_dir = std::env::temp_dir();

    // Canonicalize the temp directory for consistent comparison
    // On macOS, temp_dir() returns /var/folders/... but canonical paths are /private/var/folders/...
    let canonical_temp = temp_dir.canonicalize().unwrap_or_else(|_| temp_dir.clone());

    // Try to canonicalize the path. If it doesn't exist, check against both
    // the canonical and non-canonical temp dir prefixes
    if let Ok(canonical_path) = path.canonicalize() {
        return canonical_path.starts_with(&canonical_temp);
    }

    // Path doesn't exist - check if it starts with temp_dir or canonical temp_dir
    let path_str = path.to_string_lossy();
    let temp_str = temp_dir.to_string_lossy();
    let canonical_temp_str = canonical_temp.to_string_lossy();

    path_str.starts_with(temp_str.as_ref()) || path_str.starts_with(canonical_temp_str.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_centy_path() {
        let project_path = Path::new("/home/user/my-project");
        let centy_path = get_centy_path(project_path);

        assert_eq!(centy_path, Path::new("/home/user/my-project/.centy"));
    }

    #[test]
    fn test_get_manifest_path() {
        let project_path = Path::new("/home/user/my-project");
        let manifest_path = get_manifest_path(project_path);

        assert_eq!(
            manifest_path,
            Path::new("/home/user/my-project/.centy/.centy-manifest.json")
        );
    }

    #[test]
    fn test_centy_folder_constant() {
        assert_eq!(CENTY_FOLDER, ".centy");
    }

    #[test]
    fn test_manifest_file_constant() {
        assert_eq!(MANIFEST_FILE, ".centy-manifest.json");
    }

    #[test]
    fn test_centy_version_constant() {
        assert_eq!(CENTY_VERSION, "0.1.0");
    }

    #[test]
    fn test_now_iso_format() {
        let timestamp = now_iso();

        // Should be a valid RFC3339 timestamp
        assert!(timestamp.len() > 20, "Timestamp should be reasonably long");

        // Should contain date separators
        assert!(timestamp.contains('-'), "Should contain date separator");
        assert!(timestamp.contains(':'), "Should contain time separator");

        // Should be parseable
        let parsed = chrono::DateTime::parse_from_rfc3339(&timestamp);
        assert!(parsed.is_ok(), "Should be valid RFC3339 format");
    }

    #[test]
    fn test_get_centy_path_relative() {
        let project_path = Path::new(".");
        let centy_path = get_centy_path(project_path);

        assert_eq!(centy_path, Path::new("./.centy"));
    }

    #[test]
    fn test_paths_are_consistent() {
        let project_path = Path::new("/test");
        let centy_path = get_centy_path(project_path);
        let manifest_path = get_manifest_path(project_path);

        // Manifest path should be inside centy path
        assert!(manifest_path.starts_with(&centy_path));
    }

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
        let home_dir = dirs::home_dir().unwrap_or_else(|| Path::new("/home/user").to_path_buf());
        let test_path = home_dir.join("projects/my-project");
        assert!(!is_in_temp_dir(&test_path));
    }

    #[test]
    fn test_is_in_temp_dir_nested_temp_path() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("centy-abc123-20251215").join("subdir");
        assert!(is_in_temp_dir(&test_path));
    }
}
