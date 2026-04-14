use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::issue::IssueRef;

/// Walk up from `start` looking for a directory that contains a `.centy/` subdirectory.
///
/// Returns the first ancestor (inclusive) that has `.centy/`, or `None` if none is found.
pub(super) fn find_centy_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".centy").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None; // LLVM_COV_EXCL_LINE
        }
    }
}

/// Scan the YAML frontmatter of a Centy issue file and return the `displayNumber` value.
///
/// Frontmatter is the block between the opening `---` and the next `---` line.
/// Returns `None` if the field is absent or cannot be parsed.
fn extract_display_number(content: &str) -> Option<u32> {
    let mut in_frontmatter = false;
    for line in content.lines() {
        if line == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        if let Some(val) = line.strip_prefix("displayNumber:") {
            return val.trim().parse().ok();
        }
    }
    None
}

/// Resolve a UUID to a display number by reading `.centy/issues/<uuid>.md`.
///
/// Returns `None` when the file is absent or lacks a valid `displayNumber` field.
fn resolve_uuid_to_display_number(project_path: &Path, uuid: &str) -> Option<u32> {
    let issue_file = project_path
        .join(".centy")
        .join("issues")
        .join(format!("{uuid}.md"));
    let content = std::fs::read_to_string(issue_file).ok()?;
    extract_display_number(&content)
}

/// Parse a `centy:<number-or-uuid>` shorthand into an [`IssueRef::Local`].
///
/// Walks up from the current directory to find the nearest `.centy/` ancestor.
///
/// Both display numbers (`centy:42`) and internal UUIDs
/// (`centy:6f4853a9-3d82-4013-b909-c2d637f44541`) are accepted.  When a UUID
/// is supplied the function looks up the matching issue file to obtain the
/// display number.
///
/// # Errors
///
/// Returns an error if the identifier is neither a valid positive integer nor a
/// UUID, if the current directory cannot be determined, if no `.centy/`
/// directory is found, or if a UUID is given that cannot be resolved to a known
/// issue.
pub(super) fn parse_centy(s: &str) -> Result<IssueRef> {
    let id_str = s
        .strip_prefix("centy:")
        .expect("caller checked starts_with(\"centy:\")");

    // Fast path: display number (positive integer).
    if let Ok(display_number) = id_str.parse::<u32>() {
        // LLVM_COV_EXCL_START
        let cwd = std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Could not determine current directory: {e}"))?;
        let Some(project_path) = find_centy_root(&cwd) else {
            return Err(anyhow::anyhow!(
                "No Centy project found: could not find a .centy/ directory in {} or any parent",
                cwd.display()
            ));
        };
        return Ok(IssueRef::Local {
            project_path,
            display_number,
        });
        // LLVM_COV_EXCL_STOP
    }

    // UUID path: look up the display number from the issue file.
    if uuid::Uuid::parse_str(id_str).is_ok() {
        // LLVM_COV_EXCL_START
        let cwd = std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("Could not determine current directory: {e}"))?;
        let Some(project_path) = find_centy_root(&cwd) else {
            return Err(anyhow::anyhow!(
                "No Centy project found: could not find a .centy/ directory in {} or any parent",
                cwd.display()
            ));
        };
        let Some(display_number) = resolve_uuid_to_display_number(&project_path, id_str) else {
            return Err(anyhow::anyhow!(
                "Centy issue {id_str:?} not found in {}",
                project_path.display()
            ));
        };
        return Ok(IssueRef::Local {
            project_path,
            display_number,
        });
        // LLVM_COV_EXCL_STOP
    }

    Err(anyhow::anyhow!(
        "Invalid Centy issue identifier: {id_str:?} — expected a display number (e.g. centy:42) or a UUID"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_centy_root_in_start_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".centy")).unwrap();
        assert_eq!(find_centy_root(dir.path()), Some(dir.path().to_path_buf()));
    }

    #[test]
    fn find_centy_root_in_parent() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join(".centy")).unwrap();
        let child = root.path().join("sub").join("dir");
        fs::create_dir_all(&child).unwrap();
        assert_eq!(find_centy_root(&child), Some(root.path().to_path_buf()));
    }

    #[test]
    fn parse_centy_invalid_number() {
        let err = parse_centy("centy:abc").unwrap_err();
        assert!(err.to_string().contains("Invalid Centy issue identifier"));
    }

    #[test]
    fn extract_display_number_finds_field() {
        let content = "---\ndisplayNumber: 42\nstatus: open\n---\n# Title\n";
        assert_eq!(extract_display_number(content), Some(42));
    }

    #[test]
    fn extract_display_number_missing_field() {
        let content = "---\nstatus: open\n---\n# Title\n";
        assert_eq!(extract_display_number(content), None);
    }

    #[test]
    fn extract_display_number_with_comment_in_frontmatter() {
        let content = "---\n# This file is managed by Centy.\ndisplayNumber: 418\nstatus: in-progress\n---\n# Body\n";
        assert_eq!(extract_display_number(content), Some(418));
    }

    #[test]
    fn resolve_uuid_reads_display_number_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let issues_dir = dir.path().join(".centy").join("issues");
        fs::create_dir_all(&issues_dir).unwrap();
        let uuid = "6f4853a9-3d82-4013-b909-c2d637f44541";
        fs::write(
            issues_dir.join(format!("{uuid}.md")),
            "---\ndisplayNumber: 99\n---\n# Test\n",
        )
        .unwrap();
        assert_eq!(resolve_uuid_to_display_number(dir.path(), uuid), Some(99));
    }

    #[test]
    fn resolve_uuid_returns_none_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".centy").join("issues")).unwrap();
        assert_eq!(
            resolve_uuid_to_display_number(dir.path(), "00000000-0000-0000-0000-000000000000"),
            None
        );
    }
}
