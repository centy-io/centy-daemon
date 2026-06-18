use crate::utils::{compute_file_hash, compute_hash};
use std::path::Path;

pub fn template_hash(content: &str) -> String {
    compute_hash(content)
}

pub async fn actual_file_hash(path: &Path) -> String {
    compute_file_hash(path).await.unwrap_or_default()
}
