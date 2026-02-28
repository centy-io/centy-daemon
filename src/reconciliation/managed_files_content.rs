/// Default README content
pub const README_CONTENT: &str = r"<!-- This file is managed by Centy. Use the Centy CLI to modify it. -->
# Centy Project

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
- `archived/` - Archived items from any type
- `templates/` - Custom templates for issues and docs

## Getting Started

Create a new issue:

```bash
centy create issue
```

View all issues in the `issues/` folder.
";

/// Issues README content
pub const ISSUES_README_CONTENT: &str = r#"<!-- This file is managed by Centy. Use the Centy CLI to modify it. -->
# Issues

This folder contains project issues managed by [Centy](https://github.com/centy-io/centy-cli).

## AI Assistant Instructions

If you are an AI assistant, read this section carefully.

### Reading Issues

You can freely read issue files in this folder to understand the project's issues. Each issue contains a title, description, and metadata such as display number, status, priority, and timestamps.

### Working with Issues

1. **Modifying Issues**: Always use the `centy` CLI to modify issues. Do not directly edit issue files.

2. **Status Values**: Valid status values are defined in `config.yaml` under `statuses`. Default: `["open", "planning", "in-progress", "closed"]`

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
