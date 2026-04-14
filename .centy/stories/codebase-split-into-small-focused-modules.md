---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:53:15.918195+00:00
updatedAt: 2026-04-14T16:53:15.918195+00:00
persona: contributor
---

# Codebase split into small focused modules

## User story

As a contributor, I want the centy-daemon codebase split into small focused modules that each fit within the project's line limits, so that I can navigate, understand, and extend the code without wading through monolithic files.

## Acceptance criteria

- [ ] trait_impl.rs is split into per-domain files (each under 100 lines)
- [ ] convert_entity.rs is broken into per-domain conversion modules
- [ ] item_list filters.rs is split into per-filter-type modules
- [ ] managed_files_merge.rs separates line-based and JSON merge strategies into distinct modules
- [ ] reconciliation plan/mod.rs separates plan building, file discovery, and hashing
- [ ] init/mcp_json.rs separates file I/O from MCP config generation logic
- [ ] The create-workspace file is split into smaller cohesive files
- [ ] All split modules satisfy the Dylan max-lines-per-file and max-lines-per-function lints

## Context

The Rust Lint epic already enforced per-file and per-function line limits across most of the codebase. The remaining larger files (trait_impl.rs, convert_entity.rs, filters.rs, etc.) were deferred because they require more architectural thought. Splitting them reduces cognitive load for contributors and keeps the linter green.

## Scope

Covers pending items in epic #416: #391, #392, #393, #394, #396, #397, #116
