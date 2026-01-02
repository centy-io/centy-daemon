//! Three-way merge logic for centy sync.
//!
//! This module implements field-level merging for JSON metadata files
//! and line-level merging for Markdown content.

use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::PathBuf;

/// Result of a merge operation
#[derive(Debug, Clone)]
pub enum MergeResult<T> {
    /// Clean merge, no conflicts
    Clean(T),
    /// Conflict that needs manual resolution
    Conflict {
        /// Path where conflict file is stored
        conflict_file: PathBuf,
        /// Description of the conflict
        description: String,
    },
    /// Only ours changed, take ours
    TakeOurs(T),
    /// Only theirs changed, take theirs
    TakeTheirs(T),
}

impl<T> MergeResult<T> {
    /// Check if the merge was clean (no conflicts)
    #[must_use]
    pub fn is_clean(&self) -> bool {
        matches!(
            self,
            MergeResult::Clean(_) | MergeResult::TakeOurs(_) | MergeResult::TakeTheirs(_)
        )
    }

    /// Get the merged value if the merge was clean
    pub fn into_value(self) -> Option<T> {
        match self {
            MergeResult::Clean(v) | MergeResult::TakeOurs(v) | MergeResult::TakeTheirs(v) => {
                Some(v)
            }
            MergeResult::Conflict { .. } => None,
        }
    }
}

/// Three-way merge for JSON metadata files.
///
/// Performs field-level merging:
/// - If base == ours && base != theirs → take theirs
/// - If base == theirs && base != ours → take ours
/// - If base != ours && base != theirs && ours != theirs → conflict
/// - If ours == theirs → take either (same)
pub fn merge_json_metadata(base: &Value, ours: &Value, theirs: &Value) -> MergeResult<Value> {
    // If ours == theirs, no conflict
    if ours == theirs {
        return MergeResult::Clean(ours.clone());
    }

    // If base == ours, only theirs changed
    if base == ours {
        return MergeResult::TakeTheirs(theirs.clone());
    }

    // If base == theirs, only ours changed
    if base == theirs {
        return MergeResult::TakeOurs(ours.clone());
    }

    // Both changed - try field-level merge for objects
    if let (Value::Object(base_obj), Value::Object(ours_obj), Value::Object(theirs_obj)) =
        (base, ours, theirs)
    {
        match merge_json_objects(base_obj, ours_obj, theirs_obj) {
            Ok(merged) => MergeResult::Clean(Value::Object(merged)),
            Err(conflicting_fields) => MergeResult::Conflict {
                conflict_file: PathBuf::new(),
                description: format!("Conflicting fields: {}", conflicting_fields.join(", ")),
            },
        }
    } else {
        // Non-object values both changed differently - conflict
        MergeResult::Conflict {
            conflict_file: PathBuf::new(),
            description: "Both sides modified the value differently".to_string(),
        }
    }
}

/// Merge two JSON objects field by field
fn merge_json_objects(
    base: &Map<String, Value>,
    ours: &Map<String, Value>,
    theirs: &Map<String, Value>,
) -> Result<Map<String, Value>, Vec<String>> {
    let mut result = Map::new();
    let mut conflicts = Vec::new();

    // Collect all keys
    let all_keys: HashSet<&String> = base
        .keys()
        .chain(ours.keys())
        .chain(theirs.keys())
        .collect();

    for key in all_keys {
        let base_val = base.get(key);
        let ours_val = ours.get(key);
        let theirs_val = theirs.get(key);

        match (base_val, ours_val, theirs_val) {
            // Key only in base - deleted by both
            (Some(_), None, None) => {
                // Deleted by both, don't include
            }
            // Key only in ours - added by us
            (None, Some(v), None) => {
                result.insert(key.clone(), v.clone());
            }
            // Key only in theirs - added by them
            (None, None, Some(v)) => {
                result.insert(key.clone(), v.clone());
            }
            // Key in both ours and theirs but not base - both added
            (None, Some(ours_v), Some(theirs_v)) => {
                if ours_v == theirs_v {
                    result.insert(key.clone(), ours_v.clone());
                } else {
                    conflicts.push(key.clone());
                }
            }
            // Key in base and ours, deleted by theirs
            (Some(base_v), Some(ours_v), None) => {
                if base_v == ours_v {
                    // We didn't change it, they deleted it
                } else {
                    // We changed it, they deleted it - conflict
                    conflicts.push(key.clone());
                }
            }
            // Key in base and theirs, deleted by ours
            (Some(base_v), None, Some(theirs_v)) => {
                if base_v == theirs_v {
                    // They didn't change it, we deleted it
                } else {
                    // They changed it, we deleted it - conflict
                    conflicts.push(key.clone());
                }
            }
            // Key in all three
            (Some(base_v), Some(ours_v), Some(theirs_v)) => {
                if ours_v == theirs_v {
                    // Same change or no change
                    result.insert(key.clone(), ours_v.clone());
                } else if base_v == ours_v {
                    // Only theirs changed
                    result.insert(key.clone(), theirs_v.clone());
                } else if base_v == theirs_v {
                    // Only ours changed
                    result.insert(key.clone(), ours_v.clone());
                } else {
                    // Both changed differently
                    // Try recursive merge for nested objects
                    if let (Value::Object(base_obj), Value::Object(ours_obj), Value::Object(theirs_obj)) =
                        (base_v, ours_v, theirs_v)
                    {
                        match merge_json_objects(base_obj, ours_obj, theirs_obj) {
                            Ok(merged) => {
                                result.insert(key.clone(), Value::Object(merged));
                            }
                            Err(nested_conflicts) => {
                                for nc in nested_conflicts {
                                    conflicts.push(format!("{key}.{nc}"));
                                }
                            }
                        }
                    } else {
                        conflicts.push(key.clone());
                    }
                }
            }
            // Shouldn't happen - all keys should be in at least one map
            (None, None, None) => {}
        }
    }

    if conflicts.is_empty() {
        Ok(result)
    } else {
        Err(conflicts)
    }
}

/// Three-way merge for Markdown content.
///
/// If only one side changed, take that change.
/// If both sides changed, return a conflict.
pub fn merge_markdown(base: &str, ours: &str, theirs: &str) -> MergeResult<String> {
    // If ours == theirs, no conflict
    if ours == theirs {
        return MergeResult::Clean(ours.to_string());
    }

    // If base == ours, only theirs changed
    if base == ours {
        return MergeResult::TakeTheirs(theirs.to_string());
    }

    // If base == theirs, only ours changed
    if base == theirs {
        return MergeResult::TakeOurs(ours.to_string());
    }

    // Both changed - conflict
    MergeResult::Conflict {
        conflict_file: PathBuf::new(),
        description: "Both sides modified the markdown content".to_string(),
    }
}

/// Check if content has git conflict markers
#[must_use]
pub fn has_conflict_markers(content: &str) -> bool {
    content.contains("<<<<<<<") && content.contains("=======") && content.contains(">>>>>>>")
}

/// Resolve display number collisions after merge.
///
/// When merging, two issues might end up with the same display number.
/// This function detects and resolves such collisions by renumbering.
pub async fn resolve_display_number_collisions(
    sync_path: &std::path::Path,
) -> Result<Vec<RenumberedItem>, super::SyncError> {
    use std::collections::HashMap;
    use tokio::fs;

    let issues_path = sync_path.join("issues");
    if !issues_path.exists() {
        return Ok(Vec::new());
    }

    // Collect all display numbers and their issue IDs
    let mut display_numbers: HashMap<u32, Vec<String>> = HashMap::new();
    let mut max_display_number: u32 = 0;

    let mut entries = fs::read_dir(&issues_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            let metadata_path = entry.path().join("metadata.json");
            if metadata_path.exists() {
                if let Ok(content) = fs::read_to_string(&metadata_path).await {
                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                        if let Some(display_num) = json
                            .get("common")
                            .and_then(|c| c.get("display_number"))
                            .and_then(|n| n.as_u64())
                        {
                            let display_num = display_num as u32;
                            let issue_id = entry.file_name().to_string_lossy().to_string();
                            display_numbers
                                .entry(display_num)
                                .or_default()
                                .push(issue_id);
                            max_display_number = max_display_number.max(display_num);
                        }
                    }
                }
            }
        }
    }

    // Find and resolve collisions
    let mut renumbered = Vec::new();
    let mut next_free = max_display_number + 1;

    for (display_num, ids) in display_numbers {
        if ids.len() > 1 {
            // Keep the first one, renumber the rest
            for id in ids.into_iter().skip(1) {
                let metadata_path = issues_path.join(&id).join("metadata.json");
                if let Ok(content) = fs::read_to_string(&metadata_path).await {
                    if let Ok(mut json) = serde_json::from_str::<Value>(&content) {
                        if let Some(common) = json.get_mut("common") {
                            if let Some(obj) = common.as_object_mut() {
                                obj.insert(
                                    "display_number".to_string(),
                                    Value::Number(next_free.into()),
                                );
                            }
                        }

                        if let Ok(new_content) = serde_json::to_string_pretty(&json) {
                            if fs::write(&metadata_path, new_content).await.is_ok() {
                                renumbered.push(RenumberedItem {
                                    id,
                                    old_number: display_num,
                                    new_number: next_free,
                                });
                                next_free += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(renumbered)
}

/// Information about a renumbered item after collision resolution
#[derive(Debug, Clone)]
pub struct RenumberedItem {
    /// The issue/doc/PR ID
    pub id: String,
    /// The old display number
    pub old_number: u32,
    /// The new display number
    pub new_number: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_json_no_conflict() {
        let base = json!({"a": 1, "b": 2});
        let ours = json!({"a": 1, "b": 2});
        let theirs = json!({"a": 1, "b": 2});

        let result = merge_json_metadata(&base, &ours, &theirs);
        assert!(result.is_clean());
    }

    #[test]
    fn test_merge_json_only_theirs_changed() {
        let base = json!({"a": 1, "b": 2});
        let ours = json!({"a": 1, "b": 2});
        let theirs = json!({"a": 1, "b": 3});

        let result = merge_json_metadata(&base, &ours, &theirs);
        if let MergeResult::TakeTheirs(v) = result {
            assert_eq!(v, json!({"a": 1, "b": 3}));
        } else {
            panic!("Expected TakeTheirs");
        }
    }

    #[test]
    fn test_merge_json_only_ours_changed() {
        let base = json!({"a": 1, "b": 2});
        let ours = json!({"a": 1, "b": 3});
        let theirs = json!({"a": 1, "b": 2});

        let result = merge_json_metadata(&base, &ours, &theirs);
        if let MergeResult::TakeOurs(v) = result {
            assert_eq!(v, json!({"a": 1, "b": 3}));
        } else {
            panic!("Expected TakeOurs");
        }
    }

    #[test]
    fn test_merge_json_both_changed_same() {
        let base = json!({"a": 1, "b": 2});
        let ours = json!({"a": 1, "b": 3});
        let theirs = json!({"a": 1, "b": 3});

        let result = merge_json_metadata(&base, &ours, &theirs);
        if let MergeResult::Clean(v) = result {
            assert_eq!(v, json!({"a": 1, "b": 3}));
        } else {
            panic!("Expected Clean");
        }
    }

    #[test]
    fn test_merge_json_both_changed_different_fields() {
        let base = json!({"a": 1, "b": 2});
        let ours = json!({"a": 10, "b": 2});
        let theirs = json!({"a": 1, "b": 20});

        let result = merge_json_metadata(&base, &ours, &theirs);
        if let MergeResult::Clean(v) = result {
            assert_eq!(v, json!({"a": 10, "b": 20}));
        } else {
            panic!("Expected Clean merge of different fields");
        }
    }

    #[test]
    fn test_merge_json_conflict() {
        let base = json!({"a": 1});
        let ours = json!({"a": 2});
        let theirs = json!({"a": 3});

        let result = merge_json_metadata(&base, &ours, &theirs);
        assert!(!result.is_clean());
    }

    #[test]
    fn test_merge_markdown_no_conflict() {
        let base = "# Title\n\nContent";
        let ours = "# Title\n\nContent";
        let theirs = "# Title\n\nContent";

        let result = merge_markdown(base, ours, theirs);
        assert!(result.is_clean());
    }

    #[test]
    fn test_merge_markdown_only_theirs() {
        let base = "# Title\n\nContent";
        let ours = "# Title\n\nContent";
        let theirs = "# Title\n\nNew Content";

        let result = merge_markdown(base, ours, theirs);
        if let MergeResult::TakeTheirs(v) = result {
            assert_eq!(v, "# Title\n\nNew Content");
        } else {
            panic!("Expected TakeTheirs");
        }
    }

    #[test]
    fn test_merge_markdown_conflict() {
        let base = "# Title\n\nContent";
        let ours = "# Title\n\nOur Content";
        let theirs = "# Title\n\nTheir Content";

        let result = merge_markdown(base, ours, theirs);
        assert!(!result.is_clean());
    }

    #[test]
    fn test_has_conflict_markers() {
        assert!(has_conflict_markers("<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>>"));
        assert!(!has_conflict_markers("normal content"));
    }
}
