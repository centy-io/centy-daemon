use super::super::id::is_valid_issue_file;
use super::super::metadata::IssueFrontmatter;
use super::read::read_issue_from_frontmatter;
use super::types::{Issue, IssueCrudError};
use crate::utils::strip_centy_md_header;
use mdstore::parse_frontmatter;
use std::path::Path;
use tokio::fs;

pub(super) async fn match_entry_by_display_number(
    entry: &fs::DirEntry,
    display_number: u32,
    _issues_path: &Path,
) -> Result<Option<Issue>, IssueCrudError> {
    let file_type = entry.file_type().await?;
    let file_name_os = entry.file_name();
    let Some(name) = file_name_os.to_str() else {
        return Ok(None);
    };
    if !file_type.is_dir() && is_valid_issue_file(name) {
        let Ok(content) = fs::read_to_string(entry.path()).await else {
            return Ok(None);
        };
        let Ok((fm, _, _)) =
            parse_frontmatter::<IssueFrontmatter>(strip_centy_md_header(&content))
        else {
            return Ok(None);
        };
        if fm.display_number != display_number {
            return Ok(None);
        }
        let issue_id = name.trim_end_matches(".md");
        return Ok(Some(
            read_issue_from_frontmatter(&entry.path(), issue_id).await?,
        ));
    }
    Ok(None)
}
