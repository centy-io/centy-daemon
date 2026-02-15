# Issues

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
