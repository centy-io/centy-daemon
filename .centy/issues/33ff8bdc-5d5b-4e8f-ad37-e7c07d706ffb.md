---
# This file is managed by Centy. Use the Centy CLI to modify it.
displayNumber: 374
status: open
priority: 2
createdAt: 2026-04-01T21:46:23.089073+00:00
updatedAt: 2026-04-01T21:46:23.089073+00:00
---

# Version management: auto-migration, pinning, downgrade, and dry-run

This issue consolidates all version management features into a single tracked effort. It covers wiring up version checks into operations, auto-migration prompts, version pinning, downgrade support in the UI, dry-run/preview for migrations, and a dedicated version listing command.

Consolidates: #69, #70, #71, #72, #73, #74

---

## Background

The version management infrastructure (migrations, `VersionComparison` enum, `check_version_for_operation`) exists in the daemon but is largely disconnected from the CLI, web UI, and actual operation handlers. This issue tracks the work to make version awareness a first-class, user-facing feature.

---

## Checklist

### 1. Wire up `check_version_for_operation` (was #69)

The function at `src/version/mod.rs:37` is defined but never called.

- [ ] Call `check_version_for_operation` from key gRPC handlers in `src/server/mod.rs` (create issue, update issue, create doc, etc.)
- [ ] When `ProjectBehind`: log an info notice that an update is available
- [ ] When `ProjectAhead`: log a warning about degraded mode
- [ ] Optionally block certain operations in degraded mode or prompt for migration

**Files:** `src/version/mod.rs`, `src/server/mod.rs`

---

### 2. Auto-migration prompt when project version is behind (was #70)

Users currently get no notification when running commands on an outdated project.

- [ ] Show a notice when any command runs on a project where `project_behind` daemon version: _"Your project is at version X.X.X, daemon is at Y.Y.Y. Run 'centy update' to migrate."_
- [ ] Add `--auto-update` flag to automatically run migrations inline
- [ ] Consider blocking certain operations until migration completes (with `--force` override)
- [ ] Cache the version check to avoid repeated daemon calls per command

**Files:** `centy-cli/src/hooks/postrun.ts`, `centy-cli/src/daemon/daemon-get-project-version.ts`

---

### 3. Allow users to pin/declare project version in config.json (was #71)

The `version` field in `config.json` is only set by migrations; users cannot control it directly.

- [ ] `centy init --version X.X.X` — initialize project at a specific version
- [ ] `centy config set version X.X.X` — set version in an existing project
- [ ] Validate that the declared version is in `available_versions`; warn if not
- [ ] Add version dropdown in settings UI to pin version
- [ ] Support semver ranges (e.g. `^0.1.0`) as well as exact versions

**Files:** `centy-daemon/src/config/mod.rs`, `centy-cli/src/commands/init.ts`, `centy-cli/src/commands/config.ts`, `centy-app/components/settings/ProjectConfig.tsx`

---

### 4. Add downgrade support to web UI version management (was #72)

The web UI only surfaces the upgrade path; the CLI already supports `--target` for downgrades.

- [ ] Show the version selector regardless of comparison status (not only when `project_behind`)
- [ ] Filter `available_versions` to valid migration targets in both directions
- [ ] Add a confirmation dialog for downgrades: _"This will run X migrations in reverse"_
- [ ] Show a direction indicator (upgrade vs. downgrade) alongside the selector

```tsx
// Should: Always show, with context-aware messaging
<div className="version-management">
  <VersionSelector
    currentVersion={versionInfo.projectVersion}
    availableVersions={daemonInfo.availableVersions}
    onUpdate={handleUpdateVersion}
  />
</div>
```

**Files:** `centy-app/components/settings/ProjectConfig.tsx` (lines 476–502), `centy-app/components/settings/Settings.tsx`

---

### 5. Add migration dry-run/preview option (was #73)

Users should be able to preview migrations before committing to them.

- [ ] Add `--dry-run` flag to `centy update`; print the migration plan and exit without applying
- [ ] Add `GetMigrationPlan` RPC to `centy.proto`:
  - Request: `project_path`, `target_version`
  - Response: `current_version`, `target_version`, `direction` (upgrade/downgrade), `repeated MigrationStep`
- [ ] Implement `GetMigrationPlan` handler in `src/server/mod.rs`
- [ ] Expose migration plan info from `src/migration/registry.rs`

Example output:
```
$ centy update --dry-run
Would migrate from 0.0.0 to 0.1.0

Migrations to apply:
  1. 0.0.0 → 0.1.0: Establish version tracking system (upgrade)

No changes will be made. Run without --dry-run to apply.
```

**Files:** `centy-cli/src/commands/update.ts`, `centy-daemon/proto/centy.proto`, `centy-daemon/src/migration/registry.rs`, `centy-daemon/src/server/mod.rs`

---

### 6. Add dedicated command to list available daemon versions (was #74)

No single command shows all available versions with their descriptions.

- [ ] Add `--list` flag to `centy version` **or** create a `centy version list` subcommand
- [ ] Display all entries from `GetDaemonInfo.available_versions`, marking the current project version
- [ ] Optionally extend `DaemonInfo` or add a new RPC to include migration descriptions per version

Example output:
```
$ centy version --list
Available versions:
  0.0.0 - Unversioned (base)
  0.1.0 - Establish version tracking system  ← current

Your project: 0.1.0 | Daemon: 0.1.0 | Status: Up to date
```

**Files:** `centy-cli/src/commands/version.ts` (or new `centy-cli/src/commands/version/list.ts`)
