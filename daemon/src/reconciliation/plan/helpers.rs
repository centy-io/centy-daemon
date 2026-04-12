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
