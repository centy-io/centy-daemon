# Auto-migration prompt when project version is behind

When a user runs any centy CLI command on a project with an outdated version, they should be prompted to update.

## Current State
- User must explicitly run `centy update` to migrate
- No notification when running other commands on outdated projects
- Version mismatch is silent unless user checks `centy version`

## Expected Behavior
- When running commands on a project where `project_behind` daemon version:
  - Show a notice: "Your project is at version X.X.X, daemon is at Y.Y.Y. Run 'centy update' to migrate."
  - Optionally: Add `--auto-update` flag to automatically run migrations
  - Consider: Block certain operations until migration is complete (with override flag)

## Implementation Notes
- Could be implemented in CLI postrun hook (`centy-cli/src/hooks/postrun.ts`)
- Or as a prerun check for specific commands
- Should cache check to avoid repeated daemon calls

## Files
- `centy-cli/src/hooks/postrun.ts` - Existing hook
- `centy-cli/src/daemon/daemon-get-project-version.ts` - Version check API
