# plan.md placed in wrong location when opening with VSCode

## Bug Description

When pressing 'Open with VSCode' in the app, the plan.md file is created in the wrong location.

**Current behavior:**
- plan.md is placed at the workspace root: `{workspace_path}/plan.md`

**Expected behavior:**
- plan.md should be placed next to the issue.md file at: `.centy/issues/{issue_id}/plan.md`

## Root Cause

In `centy-daemon/src/workspace/create.rs`, the plan template is being written to the workspace root instead of the issue directory.

## Fix

Update the plan.md creation path from:
`{workspace_path}/plan.md`

To:
`.centy/issues/{issue_id}/plan.md`
