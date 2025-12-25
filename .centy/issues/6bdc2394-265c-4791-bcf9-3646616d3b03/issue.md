# Handle existing worktree folder with user options (open existing or recreate)

When opening an issue in a temp VS Code workspace and the target folder already exists, provide user-friendly options instead of just showing an error.

## Current Behavior

Shows error: “Failed to create git worktree. Try closing other VS Code windows for this project.”

## Desired Behavior

When the workspace folder already exists, prompt the user with:

1. **Open existing** - Focus the already-open VS Code window for this workspace
1. **Delete and recreate** - Remove the existing folder/worktree and create fresh

## Common Scenarios

* User opens the same issue twice on the same day
* Previous workspace wasn’t cleaned up properly
* VS Code window was closed but workspace folder persists

## Technical Context

* Error originates in `src/pr/git.rs` create_worktree() function
* Workspace path generated in `src/workspace/create.rs`
* Need to check if path exists before creating, return structured response with options
