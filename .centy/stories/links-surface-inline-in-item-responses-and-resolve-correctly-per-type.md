---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:54:16.333331+00:00
updatedAt: 2026-04-14T16:54:16.333331+00:00
persona: developer
---

# Links surface inline in item responses and resolve correctly per type

## User story

As a developer, I want item retrieval APIs to include link data inline and resolve display numbers scoped to item type, so that relationships between items are immediately visible and cross-type display number collisions produce no false errors.

## Acceptance criteria

- [ ] GetItem returns a populated links array showing related items with their titles and directions
- [ ] ListItems returns a link_count per item and supports an --include-links flag for full link data
- [ ] Display number resolution is always scoped to item type (e.g. issue:1 and plan:1 are distinct)
- [ ] SELF_LINK errors no longer appear for items of different types that share a display number
- [ ] All item types can participate as both source and target of any link
- [ ] Batch title resolution is used for link data to avoid N+1 performance issues

## Context

The link system was recently migrated from per-entity JSON files to per-link markdown files. Two gaps remain: display-number resolution is global (causing false SELF_LINK errors when e.g. issue:1 and plan:1 share a number) and link data is not surfaced in item retrieval APIs (so callers must do a separate GetLinks call).

## Scope

Covers pending items in epic #414: #360, #413
