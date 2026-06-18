use super::types::{FileInfo, ReconciliationPlan};
use crate::manifest::ManagedFileType;
use crate::utils::compute_file_hash;
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

/// Scan the .centy folder and return relative paths of all files/directories
pub fn scan_centy_folder(centy_path: &Path) -> HashSet<String> {
    let mut files = HashSet::new();
    if !centy_path.exists() {
        return files;
    }
    for entry in WalkDir::new(centy_path)
        .min_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.file_name().map(|f| f.to_str()) == Some(Some(".centy-manifest.json")) {
            continue;
        }
        if let Ok(relative) = path.strip_prefix(centy_path) {
            let mut relative_str = relative.to_string_lossy().to_string();
            if path.is_dir() {
                relative_str.push('/');
            }
            files.insert(relative_str);
        }
    }
    files
}

pub async fn collect_user_files(
    plan: &mut ReconciliationPlan,
    files_on_disk: &HashSet<String>,
    managed_paths: &HashSet<String>,
    centy_path: &Path,
) {
    for disk_path in files_on_disk {
        if !managed_paths.contains(disk_path) {
            let full_path = centy_path.join(disk_path.trim_end_matches('/'));
            let is_dir = full_path.is_dir();
            let hash = if is_dir {
                String::new()
            } else {
                compute_file_hash(&full_path).await.unwrap_or_default()
            };
            plan.user_files.push(FileInfo {
                path: disk_path.clone(),
                file_type: if is_dir {
                    ManagedFileType::Directory
                } else {
                    ManagedFileType::File
                },
                hash,
                content_preview: None,
            });
        }
    }
}
