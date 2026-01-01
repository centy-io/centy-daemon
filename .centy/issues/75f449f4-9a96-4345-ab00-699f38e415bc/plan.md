# Implementation Plan for Issue #99

**Issue ID**: 75f449f4-9a96-4345-ab00-699f38e415bc
**Title**: Auto sync centy branch

---

## Overview

Implement automatic synchronization of `.centy/` data via a dedicated orphan `centy` branch. This enables multi-machine sync, cross-branch consistency, and remote backup without polluting code branches.

**Key architectural decisions:**
- Orphan branch `centy` with no shared history with `main`
- Persistent worktree at `~/.centy/sync/{project-hash}/` always checked out to `centy` branch
- All daemon CRUD operations read/write through sync worktree
- Field-level merge for JSON, line-level for Markdown
- Transparent migration for existing projects

---

## Phase 1: Core Infrastructure

### Task 1.1: Create `src/sync/mod.rs` module structure

Create the sync module with submodules:
```
src/sync/
├── mod.rs          # Module exports and SyncError enum
├── branch.rs       # Orphan branch operations
├── worktree.rs     # Sync worktree management
└── manager.rs      # CentySyncManager orchestrator
```

**Files to create:**
- `src/sync/mod.rs` - Module definition, SyncError, re-exports
- Add `pub mod sync;` to `src/lib.rs`

**SyncError variants:**
- `NotGitRepository`
- `NoRemote`
- `WorktreeError(String)`
- `GitCommandFailed(String)`
- `MergeConflict { file: String, conflict_path: PathBuf }`
- `IoError(std::io::Error)`
- `JsonError(serde_json::Error)`

---

### Task 1.2: Implement `src/sync/branch.rs` - Orphan branch operations

**Functions to implement:**

```rust
/// Check if the centy branch exists locally or remotely
pub fn centy_branch_exists(project_path: &Path) -> Result<bool, SyncError>

/// Check if remote origin/centy exists
pub fn remote_centy_branch_exists(project_path: &Path) -> Result<bool, SyncError>

/// Create orphan centy branch with initial .centy content
/// Steps:
/// 1. git checkout --orphan centy
/// 2. git rm -rf . (remove all files)
/// 3. git add .centy/
/// 4. git commit -m "Initial centy branch"
/// 5. Return to original branch
pub fn create_orphan_centy_branch(project_path: &Path) -> Result<(), SyncError>

/// Push centy branch to origin
pub fn push_centy_branch(worktree_path: &Path) -> Result<(), SyncError>

/// Pull latest from origin/centy with rebase
pub fn pull_centy_branch(worktree_path: &Path) -> Result<PullResult, SyncError>

/// Fetch origin/centy without applying
pub fn fetch_centy_branch(project_path: &Path) -> Result<(), SyncError>
```

**PullResult enum:**
```rust
pub enum PullResult {
    UpToDate,
    FastForward,
    Merged,
    Conflict { files: Vec<String> },
}
```

---

### Task 1.3: Implement `src/sync/worktree.rs` - Sync worktree management

**Functions to implement:**

```rust
/// Get the sync worktree path for a project
/// Returns: ~/.centy/sync/{sha256(canonical_project_path)[0:16]}/
pub fn get_sync_worktree_path(project_path: &Path) -> Result<PathBuf, SyncError>

/// Check if sync worktree exists and is valid
pub fn sync_worktree_exists(project_path: &Path) -> Result<bool, SyncError>

/// Ensure sync worktree exists, creating if necessary
/// 1. Create ~/.centy/sync/ if not exists
/// 2. If worktree missing: git worktree add {path} centy
/// 3. Verify worktree is on centy branch
pub async fn ensure_sync_worktree(project_path: &Path) -> Result<PathBuf, SyncError>

/// Remove sync worktree and prune references
pub async fn remove_sync_worktree(project_path: &Path) -> Result<(), SyncError>

/// Get the centy path within sync worktree
/// Returns: {sync_worktree}/.centy/
pub fn get_sync_centy_path(project_path: &Path) -> Result<PathBuf, SyncError>
```

**Implementation notes:**
- Use existing `crate::pr::git::create_worktree` as reference
- Hash project path using SHA256, take first 16 chars for directory name
- Reuse `crate::utils::hash` module if applicable

---

### Task 1.4: Implement `src/sync/manager.rs` - CentySyncManager

**CentySyncManager struct:**

```rust
pub struct CentySyncManager {
    project_path: PathBuf,
    sync_worktree: PathBuf,
}

impl CentySyncManager {
    /// Create a new manager, ensuring sync infrastructure exists
    pub async fn new(project_path: &Path) -> Result<Self, SyncError>

    /// Get path to .centy in sync worktree
    pub fn centy_path(&self) -> PathBuf

    /// Pull latest changes before a read operation
    pub async fn pull_before_read(&self) -> Result<(), SyncError>

    /// Commit and push after a write operation
    pub async fn commit_and_push(&self, message: &str) -> Result<(), SyncError>

    /// Sync a specific file/directory from sync worktree to project
    pub async fn sync_to_project(&self, relative_path: &Path) -> Result<(), SyncError>

    /// Sync a specific file/directory from project to sync worktree
    pub async fn sync_from_project(&self, relative_path: &Path) -> Result<(), SyncError>
}
```

**Helper trait for sync-wrapped operations:**

```rust
/// Trait to wrap CRUD operations with sync
#[async_trait]
pub trait SyncWrapped {
    type Output;
    type Error;

    async fn execute_synced<F, Fut>(
        project_path: &Path,
        operation: F,
        commit_message: &str,
    ) -> Result<Self::Output, Self::Error>
    where
        F: FnOnce(PathBuf) -> Fut + Send,
        Fut: Future<Output = Result<Self::Output, Self::Error>> + Send;
}
```

---

## Phase 2: Merge & Conflict Resolution

### Task 2.1: Create `src/sync/merge.rs` - Three-way merge

**Functions to implement:**

```rust
/// Three-way merge for JSON metadata files
/// Returns merged JSON or conflict markers
pub fn merge_json_metadata(
    base: &serde_json::Value,
    ours: &serde_json::Value,
    theirs: &serde_json::Value,
) -> MergeResult<serde_json::Value>

/// Three-way merge for Markdown content
/// If both sides changed, returns conflict
pub fn merge_markdown(
    base: &str,
    ours: &str,
    theirs: &str,
) -> MergeResult<String>

/// Determine if a file has conflicts
pub fn has_conflicts(file_path: &Path) -> bool
```

**MergeResult enum:**

```rust
pub enum MergeResult<T> {
    /// Clean merge, no conflicts
    Clean(T),
    /// Conflict that needs manual resolution
    Conflict {
        /// Path where conflict file is stored
        conflict_file: PathBuf,
        /// Description of the conflict
        description: String,
    },
    /// One side unchanged, take the other
    TakeOurs(T),
    TakeTheirs(T),
}
```

**Field-level JSON merge logic:**
```rust
// For each field in the JSON:
// 1. If base == ours && base != theirs → take theirs
// 2. If base == theirs && base != ours → take ours
// 3. If base != ours && base != theirs && ours != theirs → conflict
// 4. If ours == theirs → take either (same)
```

---

### Task 2.2: Create `src/sync/conflicts.rs` - Conflict storage

**Functions to implement:**

```rust
/// Store a conflict for later resolution
pub async fn store_conflict(
    sync_path: &Path,
    item_type: &str,  // "issue", "doc", "pr"
    item_id: &str,
    conflict: ConflictInfo,
) -> Result<PathBuf, SyncError>

/// List all unresolved conflicts
pub async fn list_conflicts(sync_path: &Path) -> Result<Vec<ConflictInfo>, SyncError>

/// Get a specific conflict
pub async fn get_conflict(
    sync_path: &Path,
    conflict_id: &str,
) -> Result<Option<ConflictInfo>, SyncError>

/// Resolve a conflict with the given resolution
pub async fn resolve_conflict(
    sync_path: &Path,
    conflict_id: &str,
    resolution: ConflictResolution,
) -> Result<(), SyncError>
```

**ConflictInfo struct:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub id: String,
    pub item_type: String,
    pub item_id: String,
    pub file_path: String,
    pub created_at: String,
    pub base_content: Option<String>,
    pub ours_content: String,
    pub theirs_content: String,
}
```

**Storage location:** `.centy/.conflicts/{conflict_id}.json`

---

## Phase 3: Daemon Integration

### Task 3.1: Wrap issue CRUD operations with sync

**Modify `src/issue/crud.rs`:**

For each operation, add sync wrapper:

```rust
// Before (get_issue):
pub async fn get_issue(project_path: &Path, ...) -> Result<Issue, IssueCrudError>

// After:
pub async fn get_issue(project_path: &Path, ...) -> Result<Issue, IssueCrudError> {
    // 1. Initialize sync manager (creates worktree if needed)
    let sync = CentySyncManager::new(project_path).await?;

    // 2. Pull latest changes
    sync.pull_before_read().await?;

    // 3. Read from sync worktree
    let issue_path = sync.centy_path().join("issues").join(issue_id);
    // ... existing read logic ...
}

// For write operations (create, update, delete):
pub async fn create_issue(...) -> Result<CreateIssueResult, IssueCrudError> {
    let sync = CentySyncManager::new(project_path).await?;
    sync.pull_before_read().await?;

    // ... existing create logic (write to sync worktree) ...

    sync.commit_and_push(&format!("Create issue #{display_number}")).await?;
    Ok(result)
}
```

**Operations to wrap:**
- `get_issue` - pull before read
- `list_issues` - pull before read
- `get_issue_by_display_number` - pull before read
- `create_issue` - pull, write, commit, push
- `update_issue` - pull, write, commit, push
- `delete_issue` - pull, delete, commit, push
- `soft_delete_issue` - pull, update, commit, push
- `restore_issue` - pull, update, commit, push
- `move_issue` - pull (both projects), move, commit, push (both)
- `duplicate_issue` - pull, create, commit, push

---

### Task 3.2: Wrap docs and PR CRUD with sync

Similar pattern to issues:

**`src/docs/crud.rs`:**
- Wrap all CRUD operations with sync

**`src/pr/crud.rs`:**
- Wrap all CRUD operations with sync

---

### Task 3.3: Enhance Init to set up centy branch

**Modify `src/reconciliation/execute.rs`:**

Add centy branch initialization:

```rust
pub async fn execute_reconciliation(...) -> Result<ReconciliationResult, ...> {
    // ... existing init logic ...

    // After creating .centy directory:
    if let Err(e) = initialize_centy_sync(project_path).await {
        // Log warning but don't fail - sync is optional
        tracing::warn!("Failed to initialize centy sync: {}", e);
    }

    Ok(result)
}

async fn initialize_centy_sync(project_path: &Path) -> Result<(), SyncError> {
    // 1. Check if centy branch exists (local or remote)
    if centy_branch_exists(project_path)? {
        // Pull existing centy branch
        let sync = CentySyncManager::new(project_path).await?;
        sync.pull_before_read().await?;
        // Merge any existing local .centy with remote
        return Ok(());
    }

    // 2. Check if remote has centy branch (clone from another machine)
    if remote_centy_branch_exists(project_path)? {
        fetch_centy_branch(project_path)?;
        // Create local tracking branch
        let sync = CentySyncManager::new(project_path).await?;
        sync.pull_before_read().await?;
        return Ok(());
    }

    // 3. Create new orphan branch from existing .centy content
    create_orphan_centy_branch(project_path)?;
    push_centy_branch(project_path)?;

    // 4. Set up sync worktree
    ensure_sync_worktree(project_path).await?;

    Ok(())
}
```

---

### Task 3.4: Add gRPC endpoints for conflict management

**Add to `src/server/mod.rs`:**

```protobuf
// In proto/centy.proto:
message ListSyncConflictsRequest {
    string project_path = 1;
}

message SyncConflict {
    string id = 1;
    string item_type = 2;
    string item_id = 3;
    string file_path = 4;
    string created_at = 5;
}

message ListSyncConflictsResponse {
    repeated SyncConflict conflicts = 1;
}

message ResolveSyncConflictRequest {
    string project_path = 1;
    string conflict_id = 2;
    string resolution = 3;  // "ours" | "theirs" | "merge"
    optional string merged_content = 4;
}

message ResolveSyncConflictResponse {
    bool success = 1;
}
```

---

## Phase 4: Workspace Integration

### Task 4.1: Update workspace creation to source from sync worktree

**Modify `src/workspace/create.rs`:**

Change `copy_issue_data_to_workspace` to source from sync worktree:

```rust
async fn copy_issue_data_to_workspace(
    source_project: &Path,
    workspace_path: &Path,
    issue_id: &str,
) -> Result<(), WorkspaceError> {
    // Get sync worktree path instead of source project's .centy
    let sync_manager = CentySyncManager::new(source_project).await
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    // Pull latest before copying
    sync_manager.pull_before_read().await
        .map_err(|e| WorkspaceError::GitError(e.to_string()))?;

    let source_centy = sync_manager.centy_path();
    // ... rest of copy logic uses source_centy ...
}
```

---

### Task 4.2: Sync changes back on workspace operations

**Add to `src/workspace/cleanup.rs`:**

```rust
/// Sync any .centy changes from workspace back to sync worktree before cleanup
async fn sync_workspace_changes(
    workspace_path: &Path,
    source_project: &Path,
) -> Result<(), WorkspaceError> {
    let sync_manager = CentySyncManager::new(source_project).await?;

    // Check if workspace .centy differs from sync worktree
    let workspace_centy = workspace_path.join(".centy");
    if workspace_centy.exists() {
        // Copy changed files to sync worktree
        // Commit and push
        sync_manager.commit_and_push("Sync workspace changes").await?;
    }

    Ok(())
}
```

---

## Phase 5: Edge Cases & Resilience

### Task 5.1: Offline mode with queued operations

**Add to `src/sync/manager.rs`:**

```rust
/// Queue file for pending operations when offline
pub async fn queue_for_sync(&self, operation: SyncOperation) -> Result<(), SyncError>

/// Process queued operations when back online
pub async fn process_queue(&self) -> Result<Vec<SyncQueueResult>, SyncError>

/// Check if there are pending queued operations
pub async fn has_pending_operations(&self) -> Result<bool, SyncError>
```

**Queue storage:** `.centy/.sync-queue.json`

---

### Task 5.2: Handle missing remote

**Add checks in `CentySyncManager::new`:**

```rust
impl CentySyncManager {
    pub async fn new(project_path: &Path) -> Result<Self, SyncError> {
        // Check if remote exists
        if !has_remote_origin(project_path)? {
            // Return a "local-only" manager that skips push/pull
            return Self::new_local_only(project_path);
        }

        // ... normal initialization ...
    }

    fn new_local_only(project_path: &Path) -> Result<Self, SyncError> {
        // Create manager that operates on local .centy without sync
    }
}
```

---

### Task 5.3: Concurrent access with file locking

**Add to `src/sync/manager.rs`:**

```rust
use tokio::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Global lock map for sync operations by project path
static SYNC_LOCKS: Lazy<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

impl CentySyncManager {
    /// Acquire lock for this project's sync operations
    async fn acquire_lock(&self) -> Result<OwnedMutexGuard<()>, SyncError> {
        let mut locks = SYNC_LOCKS.lock().await;
        let lock = locks
            .entry(self.project_path.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        Ok(lock.lock_owned().await)
    }
}
```

---

### Task 5.4: Display number collision resolution

**Add to `src/sync/merge.rs`:**

```rust
/// Resolve display number collisions after merge
pub async fn resolve_display_number_collisions(
    sync_path: &Path,
) -> Result<Vec<RenumberedItem>, SyncError> {
    let issues_path = sync_path.join("issues");

    // Collect all display numbers and detect duplicates
    let mut display_numbers: HashMap<u32, Vec<String>> = HashMap::new();
    // ... scan issues ...

    // For each collision, renumber to next free number
    // UUID remains unchanged (it's the stable identifier)
    let mut renumbered = Vec::new();
    for (display_num, ids) in display_numbers {
        if ids.len() > 1 {
            // Keep first, renumber rest
            for id in ids.iter().skip(1) {
                let new_num = get_next_free_display_number(&display_numbers);
                renumber_item(sync_path, id, new_num).await?;
                renumbered.push(RenumberedItem { id: id.clone(), old_num: display_num, new_num });
            }
        }
    }

    Ok(renumbered)
}
```

---

## Dependencies

### Existing code to reuse:
- `src/pr/git.rs` - `create_worktree`, `remove_worktree`, `prune_worktrees`
- `src/utils/hash.rs` - For hashing project paths
- `src/common/org_sync.rs` - Pattern reference for sync traits
- `src/issue/reconcile.rs` - Display number logic

### External dependencies (already in Cargo.toml):
- `tokio` - Async runtime with fs operations
- `serde_json` - JSON parsing for metadata
- `thiserror` - Error types
- `async-trait` - Async traits

### New dependencies needed:
- None (all requirements already met)

---

## Edge Cases

1. **No git repository**: Skip sync gracefully, operate on local `.centy/`
2. **No remote origin**: Operate locally, log warning about backup
3. **Network failure mid-push**: Queue operation, retry on next write
4. **Concurrent writes**: Use file locking to serialize sync operations
5. **Merge conflicts**: Store in `.conflicts/`, expose via gRPC
6. **Empty centy branch**: Initialize with current `.centy/` content
7. **Corrupted worktree**: Detect and recreate automatically
8. **Display number collision**: Auto-renumber on merge, preserve UUIDs
9. **Large binary assets**: Consider gitignore or LFS for assets > 1MB
10. **Branch already exists with code**: Detect non-orphan content, abort

---

## Testing Strategy

### Unit Tests

1. **Branch operations (`branch.rs`)**
   - Test orphan branch creation
   - Test branch existence detection
   - Test push/pull operations (mock git)

2. **Worktree operations (`worktree.rs`)**
   - Test path hashing consistency
   - Test worktree creation/removal
   - Test worktree validation

3. **Merge logic (`merge.rs`)**
   - Test JSON field-level merge scenarios
   - Test Markdown merge
   - Test conflict detection

### Integration Tests

1. **Full sync cycle**
   - Create issue → verify in sync worktree
   - Update issue → verify commit and push
   - Simulate remote change → verify pull and merge

2. **Multi-machine simulation**
   - Use two worktrees as "different machines"
   - Create issue on one, verify sync to other
   - Create conflicting changes, verify resolution

3. **Workspace integration**
   - Create workspace, modify issue
   - Close workspace, verify sync back

### Manual Testing Checklist

- [ ] Fresh init creates centy branch and worktree
- [ ] Existing project with .centy migrates correctly
- [ ] Clone from remote with centy branch works
- [ ] Offline operation queues correctly
- [ ] Conflicts are stored and visible
- [ ] Conflict resolution works via CLI/gRPC
- [ ] Workspace changes sync back on close

---

## Implementation Order

1. **Phase 1** (Core Infrastructure) - Required first
   - Task 1.1 → 1.2 → 1.3 → 1.4 (sequential)

2. **Phase 2** (Merge) - Depends on Phase 1
   - Task 2.1 → 2.2 (sequential)

3. **Phase 3** (Daemon Integration) - Depends on Phase 1
   - Task 3.1 → 3.2 → 3.3 (can parallelize 3.1/3.2)
   - Task 3.4 (depends on 2.2)

4. **Phase 4** (Workspace) - Depends on Phase 3
   - Task 4.1 → 4.2 (sequential)

5. **Phase 5** (Edge Cases) - Can start after Phase 1
   - All tasks can be parallelized

---

> **Note**: After completing this plan, save it using:
> ```bash
> centy add plan 99 --file .centy/issues/75f449f4-9a96-4345-ab00-699f38e415bc/plan.md
> ```
