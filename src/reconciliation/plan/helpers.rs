use crate::manifest::ManagedFileType;
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;
use super::types::{FileInfo, ReconciliationPlan};
/// Scan the .centy folder and return relative paths of all files/directories
pub fn scan_centy_folder(centy_path: &Path) -> HashSet<String> {
    let mut files = HashSet::new();
    if !centy_path.exists() { return files; }
    for entry in WalkDir::new(centy_path).min_depth(1).into_iter().filter_map(std::result::Result::ok) {
        let path = entry.path();
        if path.file_name().map(|f| f.to_str()) == Some(Some(".centy-manifest.json")) { continue; }
        if let Ok(relative) = path.strip_prefix(centy_path) {
            let mut relative_str = relative.to_string_lossy().to_string();
            if path.is_dir() { relative_str.push('/'); }
            files.insert(relative_str);
        }
    }
    files
}
/// Build a FileInfo from a disk path that is not a managed file
#[allow(dead_code)]
pub async fn build_user_file_info(disk_path: &str, centy_path: &Path) -> FileInfo {
    use crate::utils::compute_file_hash;
    let full_path = centy_path.join(disk_path.trim_end_matches('/'));
    let is_dir = full_path.is_dir();
    let hash = if is_dir { String::new() } else { compute_file_hash(&full_path).await.unwrap_or_default() };
    FileInfo {
        path: disk_path.to_string(),
        file_type: if is_dir { ManagedFileType::Directory } else { ManagedFileType::File },
        hash,
        content_preview: None,
    }
}
/// Build a ReconciliationPlan from its constituent parts
#[allow(dead_code)]
pub fn build_plan(mut plan: ReconciliationPlan) -> ReconciliationPlan {
    plan.to_create.sort_by(|a, b| a.path.cmp(&b.path));
    plan
}
