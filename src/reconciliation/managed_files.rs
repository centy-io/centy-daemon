use crate::manifest::ManagedFileType;
use std::collections::{BTreeSet, HashMap};

/// Strategy for how a managed file should be updated when it already exists
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Merge JSON arrays (words, ignorePaths) as unions; use template's version/language;
    /// preserve user-added top-level keys
    JsonArrayMerge,
}

/// Template for a managed file
#[derive(Debug, Clone)]
pub struct ManagedFileTemplate {
    pub file_type: ManagedFileType,
    pub content: Option<String>,
    pub merge_strategy: Option<MergeStrategy>,
}

/// Default README content
const README_CONTENT: &str = r"# Centy Project

This folder is managed by [Centy](https://github.com/centy-io/centy-cli).

## Important: AI Assistant Instructions

**If you are an AI assistant working with this project:**

- **DO NOT** directly edit or create files in the `.centy/` folder
- **DO NOT** manually modify issue files, metadata, or documentation
- **ALWAYS** use the `centy` CLI commands to manage issues and documentation
- The centy cli ensures proper file structure, metadata updates, and manifest synchronization

Use the CLI commands below to interact with the centy system.

## Structure

- `issues/` - Project issues
- `docs/` - Project documentation
- `assets/` - Shared assets
- `templates/` - Custom templates for issues and docs

## Getting Started

Create a new issue:

```bash
centy create issue
```

View all issues in the `issues/` folder.
";

/// Issues README content
const ISSUES_README_CONTENT: &str = r#"# Issues

This folder contains project issues managed by [Centy](https://github.com/centy-io/centy-cli).

## AI Assistant Instructions

If you are an AI assistant, read this section carefully.

### Reading Issues

You can freely read issue files in this folder to understand the project's issues. Each issue contains a title, description, and metadata such as display number, status, priority, and timestamps.

### Working with Issues

1. **Modifying Issues**: Always use the `centy` CLI to modify issues. Do not directly edit issue files.

2. **Status Values**: Valid status values are defined in `config.json` under `allowedStates`. Default: `["open", "planning", "in-progress", "closed"]`

3. **Closing Issues**: Run `centy update issue <id> --status closed` when:
   - All requested changes have been implemented
   - Tests pass (if applicable)
   - The build succeeds (if applicable)
   - No remaining work items from the issue description

4. **When NOT to close**:
   - The task is only partially complete
   - You encountered errors or blockers
   - The user needs to review or approve before closing
   - The issue requires follow-up work

### Best Practices

- Always read the full issue content before starting work
- Check the priority to understand urgency (1 = highest priority)
- Use `centy` CLI commands for all issue modifications
"#;

/// Templates README content
const TEMPLATES_README_CONTENT: &str = r#"# Templates

This folder contains templates for creating issues and docs using [Handlebars](https://handlebarsjs.com/) syntax.

## Usage

To use a template, specify the `template` parameter when creating an issue or doc:
- Issues: Place templates in `templates/issues/` (e.g., `bug-report.md`)
- Docs: Place templates in `templates/docs/` (e.g., `api.md`)

## Available Placeholders

### Issue Templates
| Placeholder | Description |
|-------------|-------------|
| `{{title}}` | Issue title |
| `{{description}}` | Issue description |
| `{{priority}}` | Priority number (1 = highest) |
| `{{priority_label}}` | Priority label (e.g., "high", "medium", "low") |
| `{{status}}` | Issue status |
| `{{created_at}}` | Creation timestamp |
| `{{custom_fields}}` | Map of custom field key-value pairs |

### Doc Templates
| Placeholder | Description |
|-------------|-------------|
| `{{title}}` | Document title |
| `{{content}}` | Document content |
| `{{slug}}` | URL-friendly slug |
| `{{created_at}}` | Creation timestamp |
| `{{updated_at}}` | Last update timestamp |

## Handlebars Features

Templates support full Handlebars syntax:

### Conditionals
```handlebars
{{#if description}}
## Description
{{description}}
{{/if}}
```

### Loops
```handlebars
{{#each custom_fields}}
- **{{@key}}:** {{this}}
{{/each}}
```

## Example Templates

### Issue Template (`templates/issues/bug-report.md`)
```handlebars
# Bug: {{title}}

**Priority:** {{priority_label}} | **Status:** {{status}}

## Description
{{description}}

{{#if custom_fields}}
## Additional Info
{{#each custom_fields}}
- {{@key}}: {{this}}
{{/each}}
{{/if}}
```

### Doc Template (`templates/docs/api.md`)
```handlebars
---
title: "{{title}}"
slug: "{{slug}}"
---

# API: {{title}}

{{content}}
```
"#;

/// CSpell configuration content
const CSPELL_JSON_CONTENT: &str = r#"{
  "version": "0.2",
  "language": "en",
  "words": [
    "centy",
    "displayNumber",
    "createdAt",
    "updatedAt",
    "priorityLevels",
    "allowedStates",
    "stateColors",
    "priorityColors",
    "centyVersion",
    "schemaVersion"
  ],
  "ignorePaths": [
    ".centy-manifest.json"
  ]
}
"#;

/// Get the list of managed files with their templates
#[must_use]
pub fn get_managed_files() -> HashMap<String, ManagedFileTemplate> {
    let mut files = HashMap::new();

    files.insert(
        "issues/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );

    files.insert(
        "issues/README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(ISSUES_README_CONTENT.to_string()),
            merge_strategy: None,
        },
    );

    files.insert(
        "docs/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );

    files.insert(
        "assets/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );

    files.insert(
        "README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(README_CONTENT.to_string()),
            merge_strategy: None,
        },
    );

    files.insert(
        "templates/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );

    files.insert(
        "templates/issues/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );

    files.insert(
        "templates/docs/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        },
    );

    files.insert(
        "templates/README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(TEMPLATES_README_CONTENT.to_string()),
            merge_strategy: None,
        },
    );

    files.insert(
        "cspell.json".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(CSPELL_JSON_CONTENT.to_string()),
            merge_strategy: Some(MergeStrategy::JsonArrayMerge),
        },
    );

    files
}

/// Merge existing JSON content with template content using the `JsonArrayMerge` strategy.
///
/// - `words` and `ignorePaths` arrays are merged as unions, deduplicated, and sorted.
/// - `version` and `language` are taken from the template.
/// - Any additional top-level keys in the existing file are preserved.
pub fn merge_json_content(
    existing_content: &str,
    template_content: &str,
) -> Result<String, serde_json::Error> {
    let mut existing: serde_json::Value = serde_json::from_str(existing_content)?;
    let template: serde_json::Value = serde_json::from_str(template_content)?;

    let Some(existing_obj) = existing.as_object_mut() else {
        return serde_json::to_string_pretty(&template).map(|mut s| {
            s.push('\n');
            s
        });
    };
    let Some(template_obj) = template.as_object() else {
        return serde_json::to_string_pretty(&existing).map(|mut s| {
            s.push('\n');
            s
        });
    };

    // Use template's version and language (these are managed by centy)
    for key in &["version", "language"] {
        if let Some(value) = template_obj.get(*key) {
            existing_obj.insert((*key).to_string(), value.clone());
        }
    }

    // Merge array fields as unions (deduplicated, sorted)
    for key in &["words", "ignorePaths"] {
        let mut merged: BTreeSet<String> = BTreeSet::new();

        // Collect from existing
        if let Some(serde_json::Value::Array(arr)) = existing_obj.get(*key) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    merged.insert(s.to_string());
                }
            }
        }

        // Collect from template
        if let Some(serde_json::Value::Array(arr)) = template_obj.get(*key) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    merged.insert(s.to_string());
                }
            }
        }

        if !merged.is_empty() {
            let sorted_array: Vec<serde_json::Value> =
                merged.into_iter().map(serde_json::Value::String).collect();
            existing_obj.insert((*key).to_string(), serde_json::Value::Array(sorted_array));
        }
    }

    let mut output = serde_json::to_string_pretty(&existing)?;
    output.push('\n');
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_managed_files_returns_expected_files() {
        let files = get_managed_files();

        // Should have expected directories
        assert!(files.contains_key("issues/"));
        assert!(files.contains_key("docs/"));
        assert!(files.contains_key("assets/"));
        assert!(files.contains_key("templates/"));
        assert!(files.contains_key("templates/issues/"));
        assert!(files.contains_key("templates/docs/"));

        // Should have expected files
        assert!(files.contains_key("README.md"));
        assert!(files.contains_key("issues/README.md"));
        assert!(files.contains_key("templates/README.md"));
        assert!(files.contains_key("cspell.json"));
    }

    #[test]
    fn test_get_managed_files_directories_have_correct_type() {
        let files = get_managed_files();

        // All directories should have Directory type
        let directories = [
            "issues/",
            "docs/",
            "assets/",
            "templates/",
            "templates/issues/",
            "templates/docs/",
        ];
        for dir in directories {
            let template = files
                .get(dir)
                .unwrap_or_else(|| panic!("Should have {dir}"));
            assert_eq!(
                template.file_type,
                ManagedFileType::Directory,
                "Directory {dir} should have Directory type"
            );
            assert!(
                template.content.is_none(),
                "Directory {dir} should have no content"
            );
        }
    }

    #[test]
    fn test_get_managed_files_files_have_correct_type() {
        let files = get_managed_files();

        // All files should have File type
        let regular_files = [
            "README.md",
            "issues/README.md",
            "templates/README.md",
            "cspell.json",
        ];
        for file in regular_files {
            let template = files
                .get(file)
                .unwrap_or_else(|| panic!("Should have {file}"));
            assert_eq!(
                template.file_type,
                ManagedFileType::File,
                "File {file} should have File type"
            );
            assert!(
                template.content.is_some(),
                "File {file} should have content"
            );
        }
    }

    #[test]
    fn test_managed_file_template_readme_content() {
        let files = get_managed_files();
        let readme = files.get("README.md").expect("Should have README.md");

        let content = readme.content.as_ref().expect("README should have content");
        assert!(content.contains("Centy Project"));
        assert!(content.contains("AI Assistant Instructions"));
        assert!(content.contains("centy create issue"));
    }

    #[test]
    fn test_managed_file_template_issues_readme_content() {
        let files = get_managed_files();
        let readme = files
            .get("issues/README.md")
            .expect("Should have issues/README.md");

        let content = readme
            .content
            .as_ref()
            .expect("Issues README should have content");
        assert!(content.contains("Issues"));
        assert!(content.contains("AI Assistant Instructions"));
        assert!(content.contains("Reading Issues"));
        assert!(content.contains("Closing Issues"));
    }

    #[test]
    fn test_managed_file_template_templates_readme_content() {
        let files = get_managed_files();
        let readme = files
            .get("templates/README.md")
            .expect("Should have templates/README.md");

        let content = readme
            .content
            .as_ref()
            .expect("Templates README should have content");
        assert!(content.contains("Templates"));
        assert!(content.contains("Handlebars"));
        assert!(content.contains("{{title}}"));
        assert!(content.contains("{{description}}"));
    }

    #[test]
    fn test_managed_file_template_cspell_content() {
        let files = get_managed_files();
        let cspell = files.get("cspell.json").expect("Should have cspell.json");

        let content = cspell.content.as_ref().expect("cspell should have content");
        assert!(content.contains("centy"));
        assert!(content.contains("displayNumber"));
        assert!(content.contains("createdAt"));
        assert!(content.contains("allowedStates"));
    }

    #[test]
    fn test_get_managed_files_count() {
        let files = get_managed_files();

        // 6 directories + 4 files = 10 total
        assert_eq!(files.len(), 10);
    }

    #[test]
    fn test_managed_file_template_struct() {
        let template = ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some("test content".to_string()),
            merge_strategy: None,
        };

        assert_eq!(template.file_type, ManagedFileType::File);
        assert_eq!(template.content, Some("test content".to_string()));
        assert!(template.merge_strategy.is_none());
    }

    #[test]
    fn test_managed_file_template_clone() {
        let template = ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
            merge_strategy: None,
        };

        let cloned = template.clone();
        assert_eq!(cloned.file_type, ManagedFileType::Directory);
        assert!(cloned.content.is_none());
    }

    #[test]
    fn test_managed_file_template_debug() {
        let template = ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some("test".to_string()),
            merge_strategy: None,
        };

        let debug_str = format!("{template:?}");
        assert!(debug_str.contains("ManagedFileTemplate"));
        assert!(debug_str.contains("File"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_cspell_has_merge_strategy() {
        let files = get_managed_files();
        let cspell = files.get("cspell.json").expect("Should have cspell.json");
        assert_eq!(
            cspell.merge_strategy,
            Some(MergeStrategy::JsonArrayMerge),
            "cspell.json should have JsonArrayMerge strategy"
        );
    }

    #[test]
    fn test_non_cspell_files_have_no_merge_strategy() {
        let files = get_managed_files();
        for (path, template) in &files {
            if path != "cspell.json" {
                assert!(
                    template.merge_strategy.is_none(),
                    "{path} should not have a merge strategy"
                );
            }
        }
    }

    #[test]
    fn test_merge_json_content_unions_words() {
        let existing = r#"{
  "version": "0.1",
  "language": "en",
  "words": ["alpha", "centy", "custom"],
  "ignorePaths": [".centy-manifest.json"]
}"#;
        let template = r#"{
  "version": "0.2",
  "language": "en",
  "words": ["centy", "displayNumber", "createdAt"],
  "ignorePaths": [".centy-manifest.json"]
}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            words,
            vec!["alpha", "centy", "createdAt", "custom", "displayNumber"]
        );
    }

    #[test]
    fn test_merge_json_content_uses_template_version() {
        let existing = r#"{"version": "0.1", "language": "fr", "words": ["custom"]}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["centy"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["version"], "0.2");
        assert_eq!(parsed["language"], "en");
    }

    #[test]
    fn test_merge_json_content_preserves_user_keys() {
        let existing =
            r#"{"version": "0.1", "language": "en", "words": [], "flagWords": ["forbidden"]}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["centy"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["flagWords"][0], "forbidden");
    }

    #[test]
    fn test_merge_json_content_unions_ignore_paths() {
        let existing = r#"{
  "version": "0.2",
  "language": "en",
  "words": [],
  "ignorePaths": [".centy-manifest.json", "custom-path/"]
}"#;
        let template = r#"{
  "version": "0.2",
  "language": "en",
  "words": [],
  "ignorePaths": [".centy-manifest.json", "node_modules/"]
}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let paths: Vec<&str> = parsed["ignorePaths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            paths,
            vec![".centy-manifest.json", "custom-path/", "node_modules/"]
        );
    }

    #[test]
    fn test_merge_json_content_sorted_output() {
        let existing = r#"{"version": "0.2", "language": "en", "words": ["zebra", "apple"]}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["mango", "centy"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(words, vec!["apple", "centy", "mango", "zebra"]);
    }

    #[test]
    fn test_merge_json_content_deduplicates() {
        let existing =
            r#"{"version": "0.2", "language": "en", "words": ["centy", "centy", "alpha"]}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["centy", "alpha"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(words, vec!["alpha", "centy"]);
    }

    #[test]
    fn test_merge_json_content_trailing_newline() {
        let existing = r#"{"version": "0.1", "language": "en", "words": []}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": []}"#;

        let result = merge_json_content(existing, template).unwrap();
        assert!(result.ends_with('\n'));
    }
}
