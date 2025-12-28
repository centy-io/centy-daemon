# Implementation Plan: Open in Source Control Feature

## Overview

Add a source control agnostic action to open issue/PR/doc folders in their respective source control web UI (GitHub, GitLab, Bitbucket, etc.).

## Architecture

### 1. Source Control URL Builder Module (`src/source_control/`)

A new module that generates platform-specific URLs for viewing folders/files in source control.

**Structure:**
```
src/source_control/
├── mod.rs           # Public API
├── platforms.rs     # Platform-specific URL builders
└── detection.rs     # Platform detection logic
```

**Key Types:**

```rust
pub enum SourceControlPlatform {
    GitHub,
    GitLab,
    Bitbucket,
    AzureDevOps,
    Gitea,
    SelfHosted { host: String, pattern: UrlPattern },
}

pub struct SourceControlUrl {
    pub platform: SourceControlPlatform,
    pub url: String,
}

pub enum UrlPattern {
    GitHub,      // /{org}/{repo}/tree/{branch}/{path}
    GitLab,      // /{org}/{repo}/-/tree/{branch}/{path}
    Bitbucket,   // /{org}/{repo}/src/{branch}/{path}
    // etc.
}
```

**Public API:**

```rust
/// Build a URL to view a folder in source control
pub fn build_folder_url(
    project_path: &Path,
    relative_path: &str,
) -> Result<SourceControlUrl, SourceControlError>;

/// Detect which source control platform is being used
pub fn detect_platform(
    remote_url: &str,
) -> Option<SourceControlPlatform>;

/// Get the current branch name (or default to main/master)
pub fn get_current_branch_or_default(
    project_path: &Path,
) -> Result<String, SourceControlError>;
```

### 2. Integration with Entity Actions

Add "open-in-source-control" action to the existing entity actions system in `src/server/mod.rs`.

**Location:** Lines 3539-3565 (External actions section for issues)

**Logic:**
1. Check if project is a git repository
2. Get remote origin URL
3. Detect platform and check if supported
4. Enable action if all conditions met
5. Return URL as action metadata (for client to open)

**Action Definition:**

```rust
EntityAction {
    id: "open_in_source_control".to_string(),
    label: "Open in Source Control".to_string(),
    category: ActionCategory::External as i32,
    enabled: is_git_repo && has_remote && is_supported_platform,
    disabled_reason: /* contextual message */,
    destructive: false,
    keyboard_shortcut: "s".to_string(),
}
```

## Platform Support Matrix

| Platform | Host Pattern | URL Pattern | Priority |
|----------|--------------|-------------|----------|
| GitHub | `github.com` | `/{org}/{repo}/tree/{branch}/{path}` | P0 |
| GitLab | `gitlab.com` | `/{org}/{repo}/-/tree/{branch}/{path}` | P0 |
| Bitbucket | `bitbucket.org` | `/{org}/{repo}/src/{branch}/{path}` | P1 |
| Azure DevOps | `dev.azure.com` | `/{org}/_git/{repo}?path={path}&version=GB{branch}` | P1 |
| Gitea | (self-hosted) | `/{org}/{repo}/src/branch/{branch}/{path}` | P2 |

## Implementation Steps

### Phase 1: Core Module (Priority)

1. **Create `src/source_control/mod.rs`**
   - Re-export public API
   - Define error types

2. **Create `src/source_control/detection.rs`**
   - Implement `detect_platform()` using existing `parse_remote_url()`
   - Map known hosts to platforms
   - Handle self-hosted detection

3. **Create `src/source_control/platforms.rs`**
   - Implement URL builders for each platform
   - Handle special cases (branch encoding, path encoding)

4. **Create `src/source_control/builder.rs`**
   - Implement `build_folder_url()` orchestration:
     - Get git remote URL (use existing `pr::git::get_remote_origin_url`)
     - Detect platform
     - Get current branch or default
     - Build platform-specific URL

### Phase 2: Integration

5. **Update `src/server/mod.rs`**
   - Import source_control module
   - Add action to `get_entity_actions()` for Issues (line ~3565)
   - Add action for PRs (line ~3640)
   - Add action for Docs (line ~3695)

6. **Add action metadata field (optional enhancement)**
   - Extend `EntityAction` proto to include `metadata` field
   - Store generated URL in metadata for client use

### Phase 3: Testing

7. **Unit tests** in `src/source_control/`
   - Test platform detection for various URLs
   - Test URL generation for each platform
   - Test edge cases (no remote, detached HEAD, etc.)

8. **Integration tests** in `tests/`
   - Test entity actions return correct URL
   - Test with real git repositories

## Edge Cases & Error Handling

### Not a Git Repository
- **Detection**: Use existing `pr::git::is_git_repository()`
- **Action**: Disabled with reason "Not a git repository"

### No Remote Origin
- **Detection**: `get_remote_origin_url()` returns error
- **Action**: Disabled with reason "No remote origin configured"

### Unsupported Platform
- **Detection**: `detect_platform()` returns unknown/self-hosted without pattern
- **Action**: Disabled with reason "Unsupported source control platform"
- **Future**: Allow custom URL patterns in config

### Detached HEAD / No Branch
- **Detection**: `detect_current_branch()` fails
- **Fallback**: Use default branch (main/master) from `get_default_branch()`

### Special Characters in Paths
- **Handling**: URL-encode folder paths properly
- **Example**: `.centy/issues/` → `.centy%2Fissues%2F` (if needed per platform)

## URL Generation Examples

### GitHub
```
Input:  git@github.com:centy-io/centy-daemon.git
        .centy/issues/c60af527-a3f4-4cc9-baec-319bd93df8dc/

Output: https://github.com/centy-io/centy-daemon/tree/main/.centy/issues/c60af527-a3f4-4cc9-baec-319bd93df8dc
```

### GitLab
```
Input:  https://gitlab.com/myorg/myrepo.git
        .centy/docs/

Output: https://gitlab.com/myorg/myrepo/-/tree/main/.centy/docs
```

### Bitbucket
```
Input:  git@bitbucket.org:team/project.git
        .centy/prs/abc123/

Output: https://bitbucket.org/team/project/src/main/.centy/prs/abc123
```

## Configuration (Future Enhancement)

Add optional config in `.centy/config.json`:

```json
{
  "source_control": {
    "platform": "gitea",
    "url_pattern": "/{org}/{repo}/src/branch/{branch}/{path}"
  }
}
```

## Client-Side Integration

The client (CLI/Web UI) will:
1. Call `GetEntityActions` RPC for an issue/PR/doc
2. Find "open_in_source_control" action
3. If enabled, display button/menu item
4. On click, open URL in browser (from action metadata or build client-side)

**Client Flow:**
```
User clicks "Open in Source Control"
  → Client calls GetEntityActions
  → Daemon checks git repo, remote, platform
  → Returns action with URL in metadata
  → Client opens URL in default browser
```

## Security Considerations

1. **No credential exposure**: Only reads public git config (remote URLs)
2. **No git operations**: Read-only, no commits/pushes
3. **URL validation**: Ensure generated URLs are well-formed
4. **Path injection**: Sanitize relative paths to prevent escaping repo

## Performance Considerations

1. **Caching**: Consider caching remote URL detection per project
2. **Lazy evaluation**: Only detect platform when action is requested
3. **No network calls**: All operations are local git config reads

## Backward Compatibility

- No breaking changes to existing APIs
- New action is additive to entity actions
- Graceful degradation if git not available

## Testing Strategy

### Unit Tests
- Platform detection from various URL formats
- URL generation for each platform
- Error cases (no git, no remote, etc.)

### Integration Tests
- E2E test with real git repo
- Verify action appears in entity actions
- Verify URL is correct for known platforms

### Manual Testing
- Test with GitHub, GitLab, Bitbucket repos
- Test with self-hosted git
- Test edge cases (detached HEAD, no remote, etc.)

## Acceptance Criteria

✅ User can click "Open in Source Control" for any issue/PR/doc
✅ Action is disabled with clear reason if not available
✅ Works with GitHub, GitLab, Bitbucket
✅ Handles edge cases gracefully
✅ Unit tests cover all platforms
✅ No breaking changes to existing functionality

## Timeline Estimate

- **Phase 1** (Core Module): 2-3 hours
- **Phase 2** (Integration): 1-2 hours
- **Phase 3** (Testing): 1-2 hours
- **Total**: 4-7 hours

## Dependencies

**Existing Code:**
- `src/pr/git.rs` - Git operations (get remote, branch detection)
- `src/pr/remote.rs` - Remote URL parsing
- `src/server/mod.rs` - Entity actions handler

**External Crates:**
- None required (all functionality exists or is simple)

**Optional:**
- `url` crate for proper URL encoding (likely already in dependencies)
