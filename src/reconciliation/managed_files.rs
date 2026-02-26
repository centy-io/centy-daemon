use crate::manifest::ManagedFileType;
use std::collections::{BTreeSet, HashMap};
#[path = "managed_files_content.rs"] mod managed_files_content;
#[path = "managed_files_content2.rs"] mod managed_files_content2;
use managed_files_content::{ISSUES_README_CONTENT, README_CONTENT};
use managed_files_content2::{CSPELL_JSON_CONTENT, TEMPLATES_README_CONTENT};
/// Strategy for how a managed file should be updated when it already exists
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStrategy { JsonArrayMerge }
/// Template for a managed file
#[derive(Debug, Clone)]
pub struct ManagedFileTemplate {
    pub file_type: ManagedFileType,
    pub content: Option<String>,
    pub merge_strategy: Option<MergeStrategy>,
}
fn dir(files: &mut HashMap<String, ManagedFileTemplate>, key: &str) {
    files.insert(key.to_string(), ManagedFileTemplate { file_type: ManagedFileType::Directory, content: None, merge_strategy: None });
}
fn file(files: &mut HashMap<String, ManagedFileTemplate>, key: &str, content: &str, merge: Option<MergeStrategy>) {
    files.insert(key.to_string(), ManagedFileTemplate { file_type: ManagedFileType::File, content: Some(content.to_string()), merge_strategy: merge });
}
/// Get the list of managed files with their templates
#[must_use]
pub fn get_managed_files() -> HashMap<String, ManagedFileTemplate> {
    let mut files = HashMap::new();
    dir(&mut files, "issues/");
    dir(&mut files, "docs/");
    dir(&mut files, "archived/");
    dir(&mut files, "assets/");
    dir(&mut files, "templates/");
    dir(&mut files, "templates/issues/");
    dir(&mut files, "templates/docs/");
    file(&mut files, "README.md", README_CONTENT, None);
    file(&mut files, "issues/README.md", ISSUES_README_CONTENT, None);
    file(&mut files, "templates/README.md", TEMPLATES_README_CONTENT, None);
    file(&mut files, "cspell.json", CSPELL_JSON_CONTENT, Some(MergeStrategy::JsonArrayMerge));
    files
}
/// Merge existing JSON with template using `JsonArrayMerge` strategy.
pub fn merge_json_content(existing_content: &str, template_content: &str) -> Result<String, serde_json::Error> {
    let mut existing: serde_json::Value = serde_json::from_str(existing_content)?;
    let template: serde_json::Value = serde_json::from_str(template_content)?;
    let Some(existing_obj) = existing.as_object_mut() else {
        return serde_json::to_string_pretty(&template).map(|mut s| { s.push('\n'); s });
    };
    let Some(template_obj) = template.as_object() else {
        return serde_json::to_string_pretty(&existing).map(|mut s| { s.push('\n'); s });
    };
    for key in &["version", "language"] {
        if let Some(value) = template_obj.get(*key) { existing_obj.insert((*key).to_string(), value.clone()); }
    }
    for key in &["words", "ignorePaths"] {
        let mut merged: BTreeSet<String> = BTreeSet::new();
        if let Some(serde_json::Value::Array(arr)) = existing_obj.get(*key) {
            for item in arr { if let Some(s) = item.as_str() { merged.insert(s.to_string()); } }
        }
        if let Some(serde_json::Value::Array(arr)) = template_obj.get(*key) {
            for item in arr { if let Some(s) = item.as_str() { merged.insert(s.to_string()); } }
        }
        if !merged.is_empty() {
            let sorted: Vec<serde_json::Value> = merged.into_iter().map(serde_json::Value::String).collect();
            existing_obj.insert((*key).to_string(), serde_json::Value::Array(sorted));
        }
    }
    let mut output = serde_json::to_string_pretty(&existing)?;
    output.push('\n');
    Ok(output)
}
#[cfg(test)] #[path = "managed_files_tests_1.rs"] mod tests_1;
#[cfg(test)] #[path = "managed_files_tests_2.rs"] mod tests_2;
#[cfg(test)] #[path = "managed_files_tests_3.rs"] mod tests_3;
#[cfg(test)] #[path = "managed_files_tests_4.rs"] mod tests_4;
#[cfg(test)] #[path = "managed_files_tests_5.rs"] mod tests_5;
