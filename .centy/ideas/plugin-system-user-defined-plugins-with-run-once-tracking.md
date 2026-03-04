---
status: raw
priority: 3
createdAt: 2026-03-04T13:23:16.122163+00:00
updatedAt: 2026-03-04T13:23:16.122163+00:00
---

# Plugin system: user-defined plugins with run-once tracking

## Overview

Allow users to declare plugins in their project or global config file. The daemon tracks which plugins have already been executed so they only run once (unless explicitly reset or re-triggered).

## Brainstorm

### Config declaration
Users list plugins in `config.json` (or a dedicated `plugins.yaml`) under a `plugins` key:

```json
{
  "plugins": [
    { "name": "my-setup-plugin", "path": "./scripts/setup.sh" },
    { "name": "org-migrator", "url": "https://..." }
  ]
}
```

### Run-once semantics
- The daemon maintains a `plugin_runs.json` (or a registry table) that records which plugins have been invoked and their outcome (success/failure/timestamp).
- On startup or on relevant RPC calls the daemon checks: for each declared plugin, has it run successfully? If not, run it.
- Plugins that fail should be retried on next daemon start (configurable).

### Identity / fingerprinting
- A plugin is identified by its `name` + a content hash or version field.
- Changing the script content (or bumping a `version` field) resets the run flag so the new version executes.

### Execution model options
- **Shell script** - simplest; path to a .sh/.py/etc file.
- **gRPC hook** - plugin is an external process exposing a gRPC endpoint; daemon calls it.
- **HTTP webhook** - daemon POSTs to a URL (ties into issue #154).
- **WASM module** - sandboxed execution (longer term).

### Triggering
- On daemon startup (always check).
- On `centy init` for new projects.
- On explicit `centy plugins run` CLI command (force re-run).
- On config file change detection (inotify/FSEvents watch).

### State storage
- Option A: Flat file `.centy/plugin_runs.json` per project - simple, portable.
- Option B: Global registry entry in `~/.centy/registry.json` - useful for global plugins.
- Option C: Embedded SQLite - more queryable but heavier.

### Open questions
- Should plugins be scoped to a project or global?
- Should we support async plugins that signal completion via a callback?
- How to handle plugin ordering / dependencies between plugins?
- Security model: sandboxing, allowlists, signature verification?
- Should this integrate with the existing lifecycle hooks system (hooks.yaml)?
