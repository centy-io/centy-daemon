//! Atomic file write operations.
//!
//! Provides safe atomic file writing using the `tempfile` crate.
//! Temp files are automatically cleaned up on failure.

use std::io;
use std::path::Path;
use tempfile::NamedTempFile;

/// Write content to a file atomically using a temporary file.
///
/// This function:
/// - Creates a temp file in the same directory as the target (required for atomic rename)
/// - Writes the content to the temp file
/// - Atomically renames the temp file to the target path
/// - Automatically cleans up the temp file if any step fails
///
/// # Arguments
///
/// * `path` - The target file path to write to
/// * `content` - The content to write (as a string)
///
/// # Errors
///
/// Returns an `io::Error` if:
/// - The parent directory cannot be determined
/// - The temp file cannot be created
/// - Writing to the temp file fails
/// - The atomic rename fails
///
/// # Example
///
/// ```ignore
/// use crate::utils::atomic_write;
/// use std::path::Path;
///
/// // Writes atomically - file is either fully written or not modified
/// atomic_write(Path::new("/path/to/file.json"), "{\"key\": \"value\"}").await?;
/// ```
pub async fn atomic_write(path: &Path, content: &str) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Path has no parent directory"))?
        .to_path_buf();
    let target_path = path.to_path_buf();
    let content_owned = content.to_string();

    // Run synchronous tempfile operations in a blocking task
    tokio::task::spawn_blocking(move || -> io::Result<()> {
        use std::io::Write;

        // Create temp file in the same directory as target for atomic rename
        let mut temp_file = NamedTempFile::new_in(&parent)?;

        // Write content to temp file
        temp_file.write_all(content_owned.as_bytes())?;

        // Flush to ensure all data is written
        temp_file.flush()?;

        // Atomically rename temp file to target
        // This also consumes the NamedTempFile, preventing auto-deletion
        temp_file.persist(&target_path)?;

        Ok(())
    })
    .await
    .map_err(io::Error::other)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_atomic_write_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        atomic_write(&file_path, r#"{"key": "value"}"#)
            .await
            .unwrap();

        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, r#"{"key": "value"}"#);
    }

    #[tokio::test]
    async fn test_atomic_write_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // Write initial content
        std::fs::write(&file_path, "initial").unwrap();

        // Atomic overwrite
        atomic_write(&file_path, "updated").await.unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "updated");
    }

    #[tokio::test]
    async fn test_atomic_write_no_leftover_temp_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        atomic_write(&file_path, "content").await.unwrap();

        // Count files in directory - should only be the target file
        let count = std::fs::read_dir(temp_dir.path()).unwrap().count();
        assert_eq!(count, 1, "Should only have the target file, no temp files");
    }

    #[tokio::test]
    async fn test_atomic_write_fails_with_invalid_parent() {
        // Path without valid parent
        let result = atomic_write(Path::new("/nonexistent/deeply/nested/file.json"), "content").await;
        assert!(result.is_err());
    }
}
