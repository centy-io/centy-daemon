---
title: "Workspace Hooks Brainstorm"
createdAt: "2026-02-15T19:01:47.217345+00:00"
updatedAt: "2026-02-15T19:01:47.217345+00:00"
---

# Workspace Hooks Brainstorm

## Summary

The hooks system currently supports item-level CRUD operations (issue, doc, user, link, asset) with pre/post phases. Workspace creation and deletion have no hook integration.

## Brainstorm

### Operations to support

* **create** - new workspace is set up (worktree created, data copied)
* **delete** / **cleanup** - workspace is removed (expired or manually cleaned)
* **reopen** - existing workspace is reused (TTL extended, editor reopened) — open question: separate operation or variant of create with `is_reused` in context?

### Pre vs Post considerations

* **pre:workspace:create** - validate/block workspace creation (policy enforcement, max workspace count, disk space limits)
* **post:workspace:create** - environment setup after workspace is ready (install deps, configure tools, start services, notify Slack)
* **post:workspace:delete** - clean up external resources tied to the workspace

### Context data to pass

* `workspace_path`
* `source_project_path`
* `issue_id` + `issue_title` (if issue-bound, empty if standalone)
* `agent_name`
* `editor` (vscode, terminal, none)
* `is_standalone` (bool)
* `workspace_name` / `workspace_description` (standalone only)
* `is_reused` (bool)

### Use cases

1. `post:workspace:create` - run `npm install` / `cargo fetch` in the new worktree
1. `post:workspace:create` - configure git hooks or remotes
1. `post:workspace:create` - notify a channel that work started on an issue
1. `pre:workspace:create` - enforce max workspace count or disk space limits
1. `post:workspace:delete` - clean up cloud resources or CI environments

### Design questions

1. Should `workspace` be another `HookItemType`? It fits the pattern:type:operation format, but workspace is infrastructure not content — does mixing them make sense?
1. Reopen semantics — separate operation or context flag?
1. Standalone vs issue-bound — one type with context differentiation, or separate types?

## Technical context

* Hooks system: `src/hooks/` (config.rs, context.rs, executor.rs, runner.rs)
* Workspace creation: `src/workspace/orchestrator.rs`, `src/workspace/create.rs`
* Hook item types: `HookItemType` enum in `src/hooks/config.rs`
* Hook operations: `HookOperation` enum in `src/hooks/config.rs`
