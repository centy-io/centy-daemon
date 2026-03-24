use crate::manifest::ManagedFileType;
use std::collections::HashMap;
#[path = "managed_files_content.rs"]
mod managed_files_content;
#[path = "managed_files_content2.rs"]
mod managed_files_content2;
#[path = "managed_files_merge.rs"]
mod managed_files_merge;
use managed_files_content::{ISSUES_README_CONTENT, README_CONTENT};
use managed_files_content2::{CSPELL_JSON_CONTENT, HOOKS_YAML_CONTENT, TEMPLATES_README_CONTENT};
pub use managed_files_merge::merge_json_content;
/// Strategy for how a managed file should be updated when it already exists
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStrategy {
    JsonArrayMerge,
}
/// Template for a managed file
#[derive(Debug, Clone)]
pub struct ManagedFileTemplate {
    pub file_type: ManagedFileType,
    pub content: Option<String>,
    pub merge_strategy: Option<MergeStrategy>,
}
fn dir(files: &mut HashMap<String, ManagedFileTemplate>, key: &str) {
    files.insert(
        key.to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );
}
fn file(
    files: &mut HashMap<String, ManagedFileTemplate>,
    key: &str,
    content: &str,
    merge: Option<MergeStrategy>,
) {
    files.insert(
        key.to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(content.to_string()),
            merge_strategy: merge,
        },
    );
}
/// Get the list of managed files with their templates
#[must_use]
pub fn get_managed_files() -> HashMap<String, ManagedFileTemplate> {
    let mut files = HashMap::new();
    dir(&mut files, "issues/");
    dir(&mut files, "docs/");
    dir(&mut files, "archived/");
    dir(&mut files, "comments/");
    dir(&mut files, "assets/");
    dir(&mut files, "templates/");
    dir(&mut files, "templates/issues/");
    dir(&mut files, "templates/docs/");
    file(&mut files, "README.md", README_CONTENT, None);
    file(&mut files, "issues/README.md", ISSUES_README_CONTENT, None);
    file(
        &mut files,
        "templates/README.md",
        TEMPLATES_README_CONTENT,
        None,
    );
    file(
        &mut files,
        "cspell.json",
        CSPELL_JSON_CONTENT,
        Some(MergeStrategy::JsonArrayMerge),
    );
    file(&mut files, "hooks.yaml", HOOKS_YAML_CONTENT, None);
    files
}
#[cfg(test)]
#[path = "managed_files_tests_1.rs"]
mod tests_1;
#[cfg(test)]
#[path = "managed_files_tests_2.rs"]
mod tests_2;
#[cfg(test)]
#[path = "managed_files_tests_3.rs"]
mod tests_3;
#[cfg(test)]
#[path = "managed_files_tests_4.rs"]
mod tests_4;
#[cfg(test)]
#[path = "managed_files_tests_5.rs"]
mod tests_5;
