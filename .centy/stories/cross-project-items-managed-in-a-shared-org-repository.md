---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:54:05.114685+00:00
updatedAt: 2026-04-14T16:54:05.114685+00:00
persona: team-member
---

# Cross-project items managed in a shared org repository

## User story

As a team member, I want issues that span multiple projects to live in a shared organization repository and appear in any project's item list, so that I have one canonical place to track cross-cutting work instead of duplicating items per repo.

## Acceptance criteria

- [ ] ListItems merges org-wide items alongside project items when include_organization_items is set
- [ ] A cross-project ListItems RPC exists for querying the latest items across all registered projects
- [ ] Projects can be organized into groups with a persistent organization structure
- [ ] Org-level items are automatically inferred from git remote URLs or project config
- [ ] Cross-project issues are visible from any member project without manual duplication
- [ ] Org-wide docs are discoverable from any member project context

## Context

The org-wide repo feature already routes CreateItem, GetItem, UpdateItem, and DeleteItem through the org repo. The remaining gap is ListItems merging (so org items appear in every project's list view) and the broader multi-project organization primitives like grouping, auto-inference of org membership, and cross-project querying.

## Scope

Covers pending items in epic #417: #386, #354, #29, #39, #45, #61, #94, #95
