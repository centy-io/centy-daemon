---
# This file is managed by Centy. Use the Centy CLI to modify it.
displayNumber: 386
status: in-progress
priority: 1
createdAt: 2026-04-04T00:23:54.625494+00:00
updatedAt: 2026-04-04T11:46:49.120040+00:00
---

# ListItems: merge org-wide items via include_organization_items

Extend `ListItems` to include org-wide items alongside the project's own items (spec: `org-wide-centy-repo`). Depends on #384 (org repo discovery) and #385 (projects metadata field).

## Proto Changes

`ListItemsRequest` already has:

```protobuf
optional bool include_organization_items = 9;
```

The server treats a missing/unset value as `true`.

## Daemon Logic

1. Read items from the project's own `.centy/` as normal.
2. If `include_organization_items` is true (default):
   - Resolve the org repo via the registry-based discovery helper.
   - If an org repo is found, scan its items for those whose `projects` field contains the current project's slug.
   - Merge matched items into the result set.

## Acceptance Criteria

- [ ] `ListItemsRequest.include_organization_items` is wired through; missing/unset defaults to `true`
- [ ] Org-wide items filtered by current project's slug are merged into results
- [ ] When no org repo is tracked, list returns only project items (no error)
- [ ] `include_organization_items: false` skips org repo entirely
- [ ] Tests cover: default behavior, explicit true/false, no org repo present, slug filtering
