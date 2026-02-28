mod format;
mod hash;

pub use format::{format_issue_file, format_markdown, strip_centy_md_header, with_yaml_header};
pub use hash::{compute_file_hash, compute_hash};

use std::path::Path;

/// The name of the centy folder
pub const CENTY_FOLDER: &str = ".centy";

/// The name of the manifest file
pub const MANIFEST_FILE: &str = ".centy-manifest.json";

/// Current centy version (from Cargo.toml)
pub const CENTY_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    let canonical_temp = temp_dir.canonicalize().unwrap_or_else(|_| temp_dir.clone());
    if let Ok(canonical_path) = path.canonicalize() {
        return canonical_path.starts_with(&canonical_temp);
    }
    let path_str = path.to_string_lossy();
    let temp_str = temp_dir.to_string_lossy();
    let canonical_temp_str = canonical_temp.to_string_lossy();
    path_str.starts_with(temp_str.as_ref()) || path_str.starts_with(canonical_temp_str.as_ref())
}

#[cfg(test)]
#[path = "utils_tests_1.rs"]
mod tests_1;
#[cfg(test)]
#[path = "utils_tests_2.rs"]
mod tests_2;
