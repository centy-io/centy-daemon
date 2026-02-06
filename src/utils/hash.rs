use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

/// Compute SHA-256 hash of a string
#[must_use]
pub fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute SHA-256 hash of a file's contents
pub async fn compute_file_hash(path: &Path) -> Result<String, std::io::Error> {
    let content = fs::read_to_string(path).await?;
    Ok(compute_hash(&content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash("hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_compute_hash_empty() {
        let hash = compute_hash("");
        // SHA-256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_compute_hash_deterministic() {
        let hash1 = compute_hash("test content");
        let hash2 = compute_hash("test content");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_different_inputs() {
        let hash1 = compute_hash("hello");
        let hash2 = compute_hash("world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_length() {
        let hash = compute_hash("any content");
        assert_eq!(hash.len(), 64); // SHA-256 hex = 64 chars
    }

    #[tokio::test]
    async fn test_compute_file_hash() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "hello world")
            .await
            .expect("Should write");

        let hash = compute_file_hash(&file_path).await.expect("Should hash");
        assert_eq!(hash, compute_hash("hello world"));
    }

    #[tokio::test]
    async fn test_compute_file_hash_nonexistent() {
        let result = compute_file_hash(Path::new("/nonexistent/file.txt")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compute_file_hash_empty_file() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let file_path = temp_dir.path().join("empty.txt");
        tokio::fs::write(&file_path, "")
            .await
            .expect("Should write");

        let hash = compute_file_hash(&file_path).await.expect("Should hash");
        assert_eq!(hash, compute_hash(""));
    }
}
