---
displayNumber: 184
status: in-progress
priority: 2
createdAt: 2026-02-15T16:29:23.313813+00:00
updatedAt: 2026-02-15T16:31:52.423552+00:00
---

# Remove PR from init and data references

Parent: #162

Remove PR from the initialization flow and any data-layer references.

## Scope

* Remove .centy/prs/ directory creation from init command
* Remove PR-related default config (statuses, etc.) from init
* Remove PR references from reconciliation/managed files
* Remove any PR folder scanning from project loading
* Update CLI commands that reference PR (centy create pr, centy list prs, etc.)
