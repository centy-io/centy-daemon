use crate::manifest::ManagedFileType;
use std::collections::HashMap;

/// Template for a managed file
#[derive(Debug, Clone)]
pub struct ManagedFileTemplate {
    pub file_type: ManagedFileType,
    pub content: Option<String>,
}

/// Default README content
const README_CONTENT: &str = r"# Centy Project

This folder is managed by [Centy](https://github.com/centy-io/centy-cli).

## Important: LLM Instructions

**If you are an AI/LLM assistant working with this project:**

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

## LLM Instructions

If you are an AI/LLM assistant, read this section carefully.

### Issue Structure

Each issue is stored in its own folder (UUID or display number) with:

- `issue.md` - The issue content (title and description in markdown)
- `metadata.json` - Machine-readable metadata:
  - `displayNumber` - Human-friendly issue number
  - `status` - Current status (e.g., "open", "planning", "in-progress", "closed")
  - `priority` - Priority level (1 = highest)
  - `createdAt` / `updatedAt` - ISO 8601 timestamps

### Working with Issues

1. **Reading Issues**: You can freely read `issue.md` and `metadata.json` files to understand the project's issues.

2. **Modifying Issues**: Check `config.json` in the parent `.centy/` folder for the `llm` configuration section:
   - `llm.autoCloseOnComplete`: If `true`, you should update the issue status to "closed" after completing the work
   - `llm.updateStatusOnStart`: If `true`, you should update status to "in-progress" when starting work
   - `llm.allowDirectEdits`: If `true`, you may directly edit `metadata.json` to change status. If `false`, use the `centy` CLI instead

3. **Status Values**: Valid status values are defined in `config.json` under `allowedStates`. Default: `["open", "planning", "in-progress", "closed"]`

### Auto-Closing Issues

When `llm.autoCloseOnComplete` is `true` in `config.json`, you should close the issue after completing the requested work:

1. **When to close**: Close the issue when you have fully completed the task described in the issue. This includes:
   - All requested changes have been implemented
   - Tests pass (if applicable)
   - The build succeeds (if applicable)
   - No remaining work items from the issue description

2. **How to close**:
   - If `llm.allowDirectEdits` is `true`: Edit `metadata.json` and set `"status": "closed"` and update `"updatedAt"` to the current ISO 8601 timestamp
   - If `llm.allowDirectEdits` is `false`: Run `centy update issue <id> --status closed`

3. **When NOT to close**:
   - The task is only partially complete
   - You encountered errors or blockers
   - The user needs to review or approve before closing
   - The issue requires follow-up work

4. **After closing**: Inform the user that you have closed the issue and summarize what was accomplished.

### Planning Status

When an issue has status `"planning"`, a special blockquote note is embedded at the top of `issue.md`:

> **Planning Mode**: Do not implement code changes. Brainstorm with the user, create an action plan, and update this issue before transitioning to "in-progress".

**Important**: When you see this note:
- DO NOT write or modify code
- Focus on discussing the approach with the user
- Help create an action plan within the issue
- Only transition to "in-progress" when the user is ready to implement

When the status changes from "planning" to another state, this note is automatically removed.

### Best Practices

- Always read the full issue content before starting work
- Check the priority to understand urgency (1 = highest priority)
- Update status according to the project's `llm` configuration
- When closing an issue, update the `updatedAt` timestamp to the current ISO 8601 time
- Respect the planning mode when present - do not implement until transitioning out of planning
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
    "compacted",
    "priorityLevels",
    "allowedStates",
    "stateColors",
    "priorityColors",
    "autoCloseOnComplete",
    "updateStatusOnStart",
    "allowDirectEdits",
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
        },
    );

    files.insert(
        "issues/README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(ISSUES_README_CONTENT.to_string()),
        },
    );

    files.insert(
        "docs/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "assets/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(README_CONTENT.to_string()),
        },
    );

    files.insert(
        "templates/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "templates/issues/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "templates/docs/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "templates/README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(TEMPLATES_README_CONTENT.to_string()),
        },
    );

    files.insert(
        "cspell.json".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(CSPELL_JSON_CONTENT.to_string()),
        },
    );

    files
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
        assert!(content.contains("LLM Instructions"));
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
        assert!(content.contains("LLM Instructions"));
        assert!(content.contains("Issue Structure"));
        assert!(content.contains("metadata.json"));
        assert!(content.contains("autoCloseOnComplete"));
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
        };

        assert_eq!(template.file_type, ManagedFileType::File);
        assert_eq!(template.content, Some("test content".to_string()));
    }

    #[test]
    fn test_managed_file_template_clone() {
        let template = ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
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
        };

        let debug_str = format!("{template:?}");
        assert!(debug_str.contains("ManagedFileTemplate"));
        assert!(debug_str.contains("File"));
        assert!(debug_str.contains("test"));
    }
}
