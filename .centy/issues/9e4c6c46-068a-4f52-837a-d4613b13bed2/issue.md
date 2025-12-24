# Include project name in workspace folder naming

## Problem

When a workspace opens in the IDE, the folder name is currently `centy-{short_issue_id}-{timestamp}` (e.g., `centy-12345678-20231224093045`). This makes it difficult for users to identify which project is open in their IDE, especially when working on multiple projects simultaneously.

## Current Implementation

**Location:** `centy-daemon/src/workspace/create.rs:47-57`

```rust
fn generate_workspace_path(issue_id: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let short_id = if issue_id.len() > 8 {
        &issue_id[..8]
    } else {
        issue_id
    };

    let workspace_name = format!("centy-{short_id}-{timestamp}");
    std::env::temp_dir().join(workspace_name)
}
```

## Proposed Solution

Modify `generate_workspace_path()` to accept the project name and include it in the folder name:

### Option A (Recommended): `{project_name}-issue-{display_num}-{short_timestamp}`
Example: `my-app-issue-42-20231224`

- Clear project identification
- Issue number is human-readable
- Shorter timestamp (date only) keeps path manageable

### Option B: `centy-{project_name}-{short_id}-{timestamp}`
Example: `centy-my-app-12345678-20231224093045`

- Keeps the `centy-` prefix for easy identification
- Full timestamp for uniqueness

## Implementation Details

1. **Update function signature:**
   ```rust
   fn generate_workspace_path(project_name: &str, issue_display_num: u32) -> PathBuf
   ```

2. **Extract project name from path:**
   Use the last directory component of `source_project_path` or the project's configured name from `.centy/project.yaml`

3. **Sanitize project name:**
   - Replace spaces/special chars with hyphens
   - Truncate to reasonable length (e.g., 30 chars)
   - Lowercase for consistency

4. **Update callers:**
   In `create_temp_workspace()` (line 212), pass the project name:
   ```rust
   let project_name = extract_project_name(&options.source_project_path);
   let workspace_path = generate_workspace_path(&project_name, options.issue.display_number);
   ```

5. **Update tests:**
   Modify test cases in lines 262-286 to include project name parameter

## Benefits

- Users can immediately identify which project is open in the IDE title bar
- Easier to switch between multiple workspaces
- Better UX when viewing Recent Files/Workspaces in VS Code
- Helps when multiple team members are discussing which workspace they're looking at

## Considerations

- Path length limits on some OSes (Windows has 260 char limit by default)
- Need to handle edge cases: non-ASCII project names, very long names
- May want to add the issue title snippet too (e.g., `my-app-42-fix-login`)
