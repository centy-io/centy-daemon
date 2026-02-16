---
title: "Library Extraction Analysis"
createdAt: "2026-02-16T15:07:44.442398+00:00"
updatedAt: "2026-02-16T15:21:48.370287+00:00"
---

# Library Extraction Analysis

Analysis of daemon modules and what can be extracted into a standalone `centy-core` library crate.

## Current Module Map

| Module | Dependencies | Purpose |
|--------|-------------|---------|
| **common/** | none | Frontmatter parsing/generation, git helpers, metadata types, org sync |
| **utils/** | none | Paths, timestamps, SHA-256 hashing, constants |
| **config/** | utils | `.centy/config.json` management, item type definitions, config migration |
| **manifest/** | utils | Project metadata tracking (version, creation date) |
| **item/core/** | none | Core traits: `ItemCrud`, `Identifiable`, `ItemMetadata` |
| **item/generic/** | config, common, item/core, manifest, utils | Config-driven generic CRUD storage layer |
| **item/entities/issue/** | common, config, manifest, utils, registry, template, link | Issue entity (frontmatter, status, priority, assets, planning) |
| **item/entities/doc/** | common, utils | Doc entity CRUD |
| **item/operations/** | item/core | `Movable`, `Duplicable` traits |
| **item/lifecycle/** | item/core | `SoftDeletable`, `Restorable` traits |
| **item/validation/** | none | Priority and status validators |
| **item/organization/** | common | Org sync operations |
| **link/** | utils | Bidirectional entity relationships (blocks, depends-on, parent-of, etc.) |
| **search/** | utils | Custom query DSL — Pest parser, AST, evaluator, executor |
| **template/** | utils | Handlebars-based templating for item creation |
| **hooks/** | utils | Pre/post-operation bash script execution framework |
| **user/** | utils | User tracking, git contributor sync |
| **registry/** | utils, config, common | Global project registry, organization management |
| **reconciliation/** | utils, manifest, config, item | File integrity — SHA-256 divergence detection and resolution |
| **workspace/** | utils, item, config | Temporary git worktrees, editor launching, TTL-based cleanup |
| **server/** | ALL | gRPC layer — 59 RPC handlers, proto conversions, error mapping |

## Dependency Graph

```
                          SERVER (gRPC)
                              |
              +---------------+-------------------+
              |               |                   |
         WORKSPACE      RECONCILIATION       REGISTRY
              |               |                   |
              +-------+-------+                   |
                      |                           |
    +-----------------+--------------+------------+
    |                 |              |
  ITEM            TEMPLATE        CONFIG
  +-- core/           |              |
  +-- generic/        |              |
  +-- entities/       |              |
  +-- operations/     |              |
  +-- lifecycle/      |              |
  +-- validation/     |              |
  +-- organization/   |              |
    |                 |              |
    +-- LINK    SEARCH    HOOKS    USER
    |     |       |         |       |
    +-----+-------+---------+-------+
                      |
              +-------+-------+
              |               |
           COMMON          UTILS
           (foundation)    (foundation)
```

## Extraction Tiers

### Tier 1 — Strong candidates (generic, minimal coupling)

**search/** — Self-contained query engine. Has its own Pest grammar, AST, evaluator, and executor. Only depends on `utils` for path helpers. Could power search in any file-based or structured-data system.

**hooks/** — Generic lifecycle hook framework with config, context, runner, and executor. Only depends on `utils`. Reusable for any system that needs pre/post-operation scripting.

**item/core/** + **item/generic/** + **item/operations/** + **item/lifecycle/** + **item/validation/** — The trait system (`ItemCrud`, `Identifiable`, `SoftDeletable`, `Movable`, `Duplicable`) plus the config-driven generic CRUD layer. This is a reusable "file-based entity storage engine."

**common/frontmatter** — YAML frontmatter parsing and generation. Broadly useful for any Markdown-based storage system.

### Tier 2 — Good candidates (some adaptation needed)

**template/** — Handlebars templating engine. The engine is generic but the context types (`IssueTemplateContext`, `DocTemplateContext`) are centy-specific. Could be generalized with a trait-based context.

**link/** — Entity linking system with built-in relationship types and bidirectional storage. Useful for any system needing item relationships. Needs generalization of `TargetType` from enum to string.

**utils/** — Some functions are generic (`compute_hash`, `now_iso`, `format_markdown`), others are centy-specific (`get_centy_path`, `CENTY_FOLDER`). Split into generic utilities vs centy-specific constants.

**reconciliation/** — File integrity pattern using SHA-256 divergence detection. The approach is generic even if the current implementation references centy types.

### Tier 3 — Keep in daemon (project-specific)

**item/entities/issue/** — Issue-specific domain logic (planning notes, assets, display number reconciliation, org registry). Stays as a daemon-level entity that uses the extracted core.

**item/entities/doc/** — Doc-specific logic. Same treatment as issue.

**registry/** — Multi-project tracking and organization management. Tightly coupled to centy's global `~/.centy/` storage.

**workspace/** — Git worktree management, editor launching, TTL cleanup. Depends on `gwq` CLI and centy-specific workspace metadata.

**user/** — User CRUD and git contributor sync. Coupled to centy's `.centy/users.json` format.

**config/** — Centy config format and item type discovery. The `ItemTypeRegistry`/`CustomFieldDefinition` patterns are interesting but tied to `config.json` schema.

**server/** — gRPC binding layer. Always stays in the daemon.

## Proposed Library Shape

A `centy-core` crate containing:

```
centy-core/
+-- frontmatter/     # YAML frontmatter parse/generate
+-- item/
|   +-- core/        # ItemCrud, Identifiable, ItemMetadata traits
|   +-- generic/     # Config-driven generic CRUD engine
|   +-- operations/  # Movable, Duplicable traits and implementations
|   +-- lifecycle/   # SoftDeletable, Restorable traits and implementations
|   +-- validation/  # Field, status, priority validation
+-- search/          # Query DSL (Pest grammar, AST, evaluator)
+-- hooks/           # Pre/post-operation hook framework
+-- link/            # Entity relationship system
+-- template/        # Handlebars templating (with generic context trait)
+-- utils/           # Hash, timestamp, markdown formatting
```

The daemon becomes a thin layer:

```
centy-daemon/
+-- server/          # gRPC handlers, proto conversions
+-- config/          # Centy config format, item type registry
+-- manifest/        # Project manifest tracking
+-- registry/        # Multi-project + organization management
+-- workspace/       # Git worktree + editor management
+-- user/            # User tracking + git sync
+-- entities/        # Issue, Doc, PR entity implementations (use centy-core traits)
+-- reconciliation/  # File integrity (uses centy-core for hashing)
```

## What This Enables

- **CLI tools** can import `centy-core` directly without going through gRPC
- **Other projects** can use the file-based entity engine, search DSL, and hook framework independently
- **Testing** becomes easier — core logic testable without spinning up a gRPC server
- **Plugin systems** can build on the trait system without depending on the daemon