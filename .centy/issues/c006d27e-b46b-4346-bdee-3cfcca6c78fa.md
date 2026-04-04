---
# This file is managed by Centy. Use the Centy CLI to modify it.
displayNumber: 386
status: open
priority: 1
createdAt: 2026-04-04T00:23:54.625494+00:00
updatedAt: 2026-04-04T00:23:54.625494+00:00
---

# ListItems: merge org-wide items via include_org_items

Extend `ListItems` to include org-wide items alongside the project's own items (spec: `org-wide-centy-repo`). Depends on #384 (org repo discovery) and #385 (projects metadata field).

## Proto Changes

Add to `ListItemsRequest`:

```protobuf
optional bool include_org_items = N;
```

The server treats a missing/unset value as `true`.

## Daemon Logic

1. Read items from the project's own `.centy/` as normal.
2. If `include_org_items` is true (default):
   - Resolve the org repo via the registry-based discovery helper.
   - If an org repo is found, scan its items for those whose `projects` field contains the current project's slug.
   - Merge matched items into the result set.
3. Org-wide items in the response carry a `source: "org"` indicator so clients can distinguish them.

## Proto: source field

Add a `source` field to the item message (or response wrapper) to indicate origin:

```protobuf
string source = N; // "project" | "org"
```

Regular items have `source: "project"`. Org-wide items have `source: "org"`.

## Acceptance Criteria

- [ ] `ListItemsRequest` has `optional bool include_org_items`
- [ ] Missing/unset `include_org_items` defaults to `true`
- [ ] Org-wide items filtered by current project's slug are merged into results
- [ ] Each item in the response carries a `source` field (`"project"` or `"org"`)
- [ ] When no org repo is tracked, list returns only project items (no error)
- [ ] `include_org_items: false` skips org repo entirely
- [ ] Tests cover: default behavior, explicit true/false, no org repo present, slug filtering
