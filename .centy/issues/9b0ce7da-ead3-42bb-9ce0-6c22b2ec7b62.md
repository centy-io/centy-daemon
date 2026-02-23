---
displayNumber: 159
status: in-progress
priority: 2
createdAt: 2026-02-10T12:02:17.554131+00:00
updatedAt: 2026-02-22T23:38:03.537108+00:00
---

# Flatten config to dot-separated keys (VS Code style)

Switch config.json from nested objects to dot-separated flat keys, similar to VS Code settings.

## Current format (nested)

````json
{
  "llm": {
    "autoCloseOnComplete": false,
    "updateStatusOnStart": false,
    "allowDirectEdits": false,
    "defaultWorkspaceMode": 0
  }
}
````

## New format (flat, dot-separated)

````json
{
  "llm.autoCloseOnComplete": false,
  "llm.updateStatusOnStart": false,
  "llm.allowDirectEdits": false,
  "llm.defaultWorkspaceMode": 0
}
````

## Requirements

1. **Support both formats** — the daemon must accept both the old nested format and the new flat dot-separated format during the transition period
1. **Auto-convert on read** — when the daemon reads a config with the old nested format, it should automatically convert it to the new flat format and write it back to disk
1. **Converter in a separate file** — the migration/conversion logic should live in its own file (e.g. `src/config/migrate.rs`) so it can be cleanly removed once all projects have migrated
1. **Deprecation warning** — log a warning when the old nested format is detected
1. **Update serde model** — `CentyConfig` should deserialize from either format but always serialize to the new flat format
1. **Apply to all nested config sections** — not just `llm`, but any future nested sections should follow this pattern

## Affected files

* `src/config/mod.rs` — update `CentyConfig` / `LlmConfig` serde model
* `src/config/migrate.rs` (new) — conversion logic from nested to flat format
* `read_config()` — call migration before deserializing
* Proto/gRPC layer — ensure flat keys are properly mapped
* E2E tests — verify both formats are accepted and old format is auto-converted
