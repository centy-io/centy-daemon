---
createdAt: 2026-02-25T22:50:02.146900+00:00
updatedAt: 2026-03-05T16:34:16.713574+00:00
customFields:
  acceptance-criteria: User can attach one or more repos to any item (issue, epic, bug, or custom type). Each repo shows its own status independently. Item dashboard shows aggregate progress. Workspace open command respects all linked repos.
  persona: alex-multi-repo-feature-owner
---

# Connect an item to multiple repos and track per-repo progress

## Context

Alex is shipping a feature that requires coordinated changes across 3 repos (e.g. daemon, CLI, web). Right now they have to create duplicate items in each repo or rely on manual cross-references like 'also see org/cli#42'.

## The Job To Be Done

> "I'm shipping a feature that requires coordinated changes in 3 repos. I want **one item** (issue, epic, bug, or any custom type) that represents the whole thing, with per-repo progress visible."

## Concrete Scenarios

- **Breaking proto change** — daemon changes a proto field → CLI must update generated types. One item, two repos.
- **New item type** — daemon adds handler, CLI adds command, web adds UI. Three repos, one feature item.
- **Cross-cutting refactor** — rename a concept that touches daemon, CLI, docs, web. All need coordinated PRs before any can ship.
- **Org item today** — Alex creates an org-level item of any type (stored in `~/.centy/orgs/{slug}/issues/`). The daemon auto-syncs a read-only copy into every project in the org, each with its own local display number. Updates to the org item propagate to all copies. But there is no per-repo status on the org item itself — it is a flat shared item. Alex cannot tell from the org item alone which repos have completed their work and which are still open.
- **Org doc today** — Alex marks a doc as `isOrgDoc: true` when creating it through a project-level endpoint. The daemon syncs it into every other org project by slug. A rename in one project cascades everywhere. But the doc has no structured link to repo-level items, so Alex has to manually mention it in item bodies — there is no way to surface it automatically from the context of a specific repo's work.

## What Centy Could Offer

- `repos` field on any item (issue, epic, bug, or custom type) listing each connected repo
- Per-repo status (open / in-progress / done)
- Workspace command creates worktrees in each linked repo simultaneously
- Item only auto-closeable when all per-repo work is resolved
- `centy list <type> --repo cli` filters by repo context
