# Sync should update existing users with missing data

## Problem
When running git sync, existing users are skipped entirely. If a user was created before git_usernames was populated (or has other missing data), running sync again won't update them.

## Proposed Enhancement
Sync should optionally update existing users with missing/partial data.

## Options to Consider

### Option 1: Update only empty fields
- If user exists but `git_usernames` is empty, populate it from git history
- Don't overwrite if user already has git_usernames
- Pros: Safe, non-destructive
- Cons: Won't add new git aliases

### Option 2: Merge git_usernames
- Append new git author names to existing git_usernames array
- Deduplicate to avoid duplicates
- Pros: Comprehensive, captures all git aliases
- Cons: May accumulate stale usernames

### Option 3: Flag-controlled behavior
- `--update-existing` flag to enable updating existing users
- `--merge-usernames` vs `--replace-usernames` sub-options
- Pros: User control
- Cons: More complexity

### Option 4: Separate command
- `centy users update-from-git` separate from sync
- Only updates existing users, doesn't create new ones
- Pros: Clear separation of concerns
- Cons: Extra command to remember

## Recommendation
Start with Option 1 (update only empty fields) as default behavior, with Option 3 flags for advanced use cases.

## Implementation Notes
- Modify `sync_users()` in `src/user/sync.rs`
- Check existing users' `git_usernames` field
- If empty, update with contributor name
- Add to `skipped` → `updated` tracking in result
