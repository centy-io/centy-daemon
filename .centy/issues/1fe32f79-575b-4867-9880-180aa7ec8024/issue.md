# Validate project path is absolute before processing

The daemon should validate that the `projectPath` received in requests (Init, GetReconciliationPlan, ExecuteReconciliation, etc.) is an absolute path.

**Current behavior:**
If a relative or invalid path is provided (e.g., just a folder name like `my-project` instead of `/Users/foo/my-project`), the daemon may silently fail or behave unexpectedly.

**Expected behavior:**
Return a clear error message when an invalid/relative path is provided, such as:
- "Invalid path: projectPath must be an absolute path"
- Include the invalid path in the error message for debugging

**Context:**
This is a defensive measure. The UI should prevent invalid paths, but the daemon should also validate to catch edge cases and provide better error messages.
