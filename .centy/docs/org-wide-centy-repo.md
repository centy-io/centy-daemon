---
# This file is managed by Centy. Use the Centy CLI to modify it.
createdAt: 2026-04-03T08:38:14.470466+00:00
updatedAt: 2026-04-03T08:38:14.470466+00:00
---

# Org-Wide .centy Repo

# Org-Wide `.centy` Repo

**Date:** 2026-04-03
**Status:** Approved

## Overview

Introduce a special git repository named `.centy` that serves as the org-wide storage for items that span multiple repos. Every item that belongs to the whole organization — rather than a single project — lives here. Individual project repos remain unchanged; they gain the ability to create items directly into the org repo, and to read org-wide items as part of their normal item listings.

## The `.centy` Repo Structure

The `.centy` repo is a plain git repository. Its **root directory is the data root** — there is no nested `.centy/` subfolder inside it. Items are stored at the top level, matching the same layout used inside a regular project's `.centy/` folder:

```
~/dev/acme/.centy/          ← git repo root AND data root
  issues/
    <id>/
      issue.md
      metadata.json
  config.json
  organization.json
```

Regular projects store data one level in:

```
~/dev/acme/frontend/
  .centy/                   ← data root is here
    issues/
    config.json
```

The daemon resolves the data root differently depending on whether it is operating on a regular project or the org repo.

## Org Repo Discovery

Given a `project_path`, the daemon resolves the org repo path as follows:

1. Read `.centy/config.json` for an `org_repo_path` field. If present, use that path as the org repo data root.
2. If missing, check for a sibling directory named `.centy` at the parent level (e.g., `~/dev/acme/frontend` → `~/dev/acme/.centy`).
3. If neither exists, no org repo is available. Cross-repo operations are silently skipped (for reads) or return a clear error (for writes).

The resolved path is computed per-request. It is not cached or persisted to the global registry.

## Item Metadata: `projects` Field

Org-wide items carry a `projects` field in their metadata — a list of project slugs that the item is relevant to:

```json
{
  "projects": ["frontend"]
}
```

At creation time, the daemon automatically sets `projects` to `[<originating_project_slug>]`. Managing the full list (adding/removing projects) is out of scope for this iteration.

## Proto Changes

### `CreateItemRequest`

Add `org_wide: bool`. When true, the item is written to the org repo instead of the project's own `.centy/`.

```protobuf
bool org_wide = N;
```

### `ListItemsRequest`

Add `include_org_items: optional bool`. The server treats missing or unset as `true`.

```protobuf
optional bool include_org_items = N;
```

## Daemon Logic

### `CreateItem` with `org_wide: true`

1. Resolve the org repo path for the given `project_path`.
2. If no org repo is found, return a clear error.
3. Derive the originating project's slug from `project_path`.
4. Write the item to the org repo's data root (e.g., `<org_repo>/issues/<id>/`).
5. Set `projects: [<slug>]` in the item's metadata.

### `ListItems`

1. Read items from the project's own `.centy/` as normal.
2. If `include_org_items` is true (default):
   - Resolve the org repo path.
   - Scan org repo items for those whose `projects` field contains the current project's slug.
   - Merge matched items into the result set.
3. Org-wide items in the response carry a `source: "org"` indicator so clients can render them as read-only.

### `GetItem`

1. Look in the project's own `.centy/` first.
2. If not found and an org repo exists, look in the org repo.

### Write operations on org-wide items from a non-org project

`UpdateItem` and `DeleteItem` called on an org-wide item from outside the `.centy` repo return a clear error:

> "This item belongs to the org repo and cannot be modified from this project."

## Out of Scope (This Iteration)

- Adding/removing projects from an item's `projects` field after creation
- Reading org-wide items that are not linked to the current project
- Write access to org-wide items from non-org projects
- Caching or pre-indexing of the org repo
