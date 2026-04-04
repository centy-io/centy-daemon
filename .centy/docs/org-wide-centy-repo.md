---
# This file is managed by Centy. Use the Centy CLI to modify it.
createdAt: 2026-04-03T08:38:14.470466+00:00
updatedAt: 2026-04-04T00:40:57.586809+00:00
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
    <id>.md
  config.json
  organization.json
```

Regular projects store data one level in:

```
~/dev/acme/frontend/
  .centy/                   ← data root is here
    issues/
      <id>.md
    config.json
```

The daemon resolves the data root differently depending on whether it is operating on a regular project or the org repo.

## Org Repo Discovery

The org repo is a normal tracked project whose path ends in `/.centy`. Given a `project_path`, the daemon discovers the org repo as follows:

1. Determine the project's organization slug from the registry.
2. Scan tracked projects for one in the same org whose path ends in `/.centy`.
3. Return it, or `None` if not found. Cross-repo operations are silently skipped (for reads) or return a clear error (for writes).

Discovery is computed per-request. It is not cached.

## Item Metadata: `projects` Field

Items carry a `projects` field in their frontmatter — a list of project slugs that the item is associated with:

```markdown
---
projects: ["frontend", "backend"]
---
```

## Proto Changes

### `CreateItemRequest`

Add `projects: repeated string`. This field serves two purposes:

1. **Routing**: if `len(projects) > 1`, the item is written to the org repo instead of the project's own `.centy/`.
2. **Association**: the value is stored as-is in the item's `projects` frontmatter field.

```protobuf
repeated string projects = N;
```

If `projects` is empty or has a single entry, the item is written to the project's own `.centy/` as normal.

### `ListItemsRequest`

Add `include_organization_items: optional bool`. The server treats missing or unset as `true`.

```protobuf
optional bool include_organization_items = N;
```

### `GenericItemMetadata`

Add `projects: repeated string` to carry associated project slugs:

```protobuf
repeated string projects = N;
```

## Daemon Logic

### `CreateItem`

1. If `len(projects) > 1`:
   - Discover the org repo via the registry.
   - If no org repo is tracked, return a clear error.
   - Write the item to the org repo's data root.
2. Otherwise, write to the project's own `.centy/` as normal.
3. Store `projects` in the item's frontmatter.

Data-root resolution for the org repo: for a project whose path ends in `/.centy`, the data root is the repo root itself (no nested `.centy/` subfolder).

### `ListItems`

1. Read items from the project's own `.centy/` as normal.
2. If `include_organization_items` is true (default):
   - Discover the org repo via the registry.
   - Scan org repo items for those whose `projects` frontmatter field contains the current project's slug.
   - Merge matched items into the result set.

### `GetItem`

1. Look in the project's own `.centy/` first.
2. If not found and an org repo is tracked, look in the org repo.

### `UpdateItem` / `DeleteItem`

Write operations on org-wide items are fully supported. The daemon checks whether the item lives in the org repo and routes the write there automatically — the caller does not need to know or specify where the item is stored.

## Out of Scope (This Iteration)

- Adding/removing projects from an item's `projects` field after creation
- Reading org-wide items that are not linked to the current project
- Caching or pre-indexing of the org repo
