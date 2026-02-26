use std::path::Path;
use tokio::fs;
/// Count issues (both new .md format and old folder format)
pub async fn count_issues(path: &Path) -> Result<u32, std::io::Error> {
    if !path.exists() {
        return Ok(0);
    }
    let mut count: u32 = 0;
    let mut entries = fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        let is_new_format = file_type.is_file() && is_uuid_file(&name);
        let is_old_format = file_type.is_dir() && is_uuid_folder(&name);
        if is_new_format || is_old_format {
            count = count.saturating_add(1);
        }
    }
    Ok(count)
}
fn is_uuid_file(name: &str) -> bool {
    name.strip_suffix(".md")
        .is_some_and(|base| uuid::Uuid::parse_str(base).is_ok())
}
fn is_uuid_folder(name: &str) -> bool {
    uuid::Uuid::parse_str(name).is_ok()
}
/// Count markdown files in a path (for counting docs)
pub async fn count_md_files(path: &Path) -> Result<u32, std::io::Error> {
    if !path.exists() {
        return Ok(0);
    }
    let mut count: u32 = 0;
    let mut entries = fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        if file_type.is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    count = count.saturating_add(1);
                }
            }
        }
    }
    Ok(count)
}
