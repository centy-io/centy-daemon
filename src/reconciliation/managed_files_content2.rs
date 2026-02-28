/// Hooks YAML template content
pub const HOOKS_YAML_CONTENT: &str = "# This file is managed by Centy. Use the Centy CLI to modify it.\n# Centy Hooks \u{2014} https://docs.centy.io/hooks\n#\n# Example hook:\n# hooks:\n#   - event: issue.created\n#     run: echo \"Issue created: $CENTY_ITEM_TITLE\"\n";

/// CSpell configuration content
pub const CSPELL_JSON_CONTENT: &str = r#"{
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
/// Templates README content (part 1 - header and placeholders)
pub const TEMPLATES_README_CONTENT: &str = concat!(
    "<!-- This file is managed by Centy. Use the Centy CLI to modify it. -->\n",
    "# Templates\n\n",
    "This folder contains templates for creating issues and docs using ",
    "[Handlebars](https://handlebarsjs.com/) syntax.\n\n",
    "## Usage\n\n",
    "To use a template, specify the `template` parameter when creating an issue or doc:\n",
    "- Issues: Place templates in `templates/issues/` (e.g., `bug-report.md`)\n",
    "- Docs: Place templates in `templates/docs/` (e.g., `api.md`)\n\n",
    "## Available Placeholders\n\n",
    "### Issue Templates\n",
    "| Placeholder | Description |\n",
    "|-------------|-------------|\n",
    "| `{{title}}` | Issue title |\n",
    "| `{{description}}` | Issue description |\n",
    "| `{{priority}}` | Priority number (1 = highest) |\n",
    "| `{{priority_label}}` | Priority label (e.g., \"high\", \"medium\", \"low\") |\n",
    "| `{{status}}` | Issue status |\n",
    "| `{{created_at}}` | Creation timestamp |\n",
    "| `{{custom_fields}}` | Map of custom field key-value pairs |\n\n",
    "### Doc Templates\n",
    "| Placeholder | Description |\n",
    "|-------------|-------------|\n",
    "| `{{title}}` | Document title |\n",
    "| `{{content}}` | Document content |\n",
    "| `{{slug}}` | URL-friendly slug |\n",
    "| `{{created_at}}` | Creation timestamp |\n",
    "| `{{updated_at}}` | Last update timestamp |\n\n",
    "## Handlebars Features\n\n",
    "Templates support full Handlebars syntax, including:\n\n",
    "### Conditionals\n\n",
    "Use `{{#if field}}` to conditionally include content:\n\n",
    "```\n",
    "{{#if description}}\n",
    "**Description:** {{description}}\n",
    "{{/if}}\n",
    "```\n\n",
    "### Loops\n\n",
    "Use `{{#each items}}` to iterate over lists:\n\n",
    "```\n",
    "{{#each custom_fields}}\n",
    "- {{@key}}: {{this}}\n",
    "{{/each}}\n",
    "```\n",
);
