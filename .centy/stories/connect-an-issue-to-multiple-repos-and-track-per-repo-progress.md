---
createdAt: 2026-02-25T22:50:02.146900+00:00
updatedAt: 2026-02-25T22:50:02.146900+00:00
customFields:
  acceptance-criteria: User can attach one or more repos to an issue. Each repo shows its own status independently. Issue dashboard shows aggregate progress. Workspace open command respects all linked repos.
  persona: alex-multi-repo-feature-owner
---

# Connect an issue to multiple repos and track per-repo progress

## Context

Alex is shipping a feature that requires coordinated changes across 3 repos (e.g. daemon, CLI, web). Right now they have to create duplicate issues in each repo or rely on manual cross-references like 'also see org/cli#42'.

## The Job To Be Done

> "I'm shipping a feature that requires coordinated changes in 3 repos. I want **one issue** that represents the whole thing, with per-repo progress visible."

## Concrete Scenarios

- **Breaking proto change** — daemon changes a proto field → CLI must update generated types. One issue, two repos.
- **New item type** — daemon adds handler, CLI adds command, web adds UI. Three repos, one feature issue.
- **Cross-cutting refactor** — rename a concept that touches daemon, CLI, docs, web. All need coordinated PRs before any can ship.

## What Centy Could Offer

- `repos` field on an issue listing each connected repo
- Per-repo status (open / in-progress / done)
- Workspace command creates worktrees in each linked repo simultaneously
- Issue only auto-closeable when all per-repo items are resolved
- `centy list issue --repo cli` filters by repo context
