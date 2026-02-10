# Issues

This folder contains project issues managed by [Centy](https://github.com/centy-io/centy-cli).

## LLM Instructions

If you are an AI/LLM assistant, read this section carefully.

### Reading Issues

You can freely read issue files in this folder to understand the project's issues. Each issue contains a title, description, and metadata such as display number, status, priority, and timestamps.

### Working with Issues

1. **Modifying Issues**: Check `config.json` in the parent `.centy/` folder for the `llm` configuration section:
   - `llm.autoCloseOnComplete`: If `true`, you should update the issue status to "closed" after completing the work
   - `llm.updateStatusOnStart`: If `true`, you should update status to "in-progress" when starting work
   - `llm.allowDirectEdits`: If `true`, you may directly edit issue files to change status. If `false`, use the `centy` CLI instead

2. **Status Values**: Valid status values are defined in `config.json` under `allowedStates`. Default: `["open", "planning", "in-progress", "closed"]`

### Auto-Closing Issues

When `llm.autoCloseOnComplete` is `true` in `config.json`, you should close the issue after completing the requested work:

1. **When to close**: Close the issue when you have fully completed the task described in the issue. This includes:
   - All requested changes have been implemented
   - Tests pass (if applicable)
   - The build succeeds (if applicable)
   - No remaining work items from the issue description

2. **How to close**: Run `centy update issue <id> --status closed`

3. **When NOT to close**:
   - The task is only partially complete
   - You encountered errors or blockers
   - The user needs to review or approve before closing
   - The issue requires follow-up work

4. **After closing**: Inform the user that you have closed the issue and summarize what was accomplished.

### Best Practices

- Always read the full issue content before starting work
- Check the priority to understand urgency (1 = highest priority)
- Update status according to the project's `llm` configuration
