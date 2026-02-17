---
title: "Daemon Architecture Overview"
createdAt: "2026-02-16T15:04:56.752536+00:00"
updatedAt: "2026-02-17T10:16:14.633056+00:00"
---

# Daemon Architecture Overview

# Daemon Architecture Overview

ASCII diagram covering the full architecture of centy-daemon: clients, gRPC server, domain layer, storage (powered by mdstore), and integrations.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            CENTY DAEMON ARCHITECTURE                            │
│                    Local-First File-Based Project Management                    │
└─────────────────────────────────────────────────────────────────────────────────┘

   CLIENTS
  ─────────
  ┌──────────┐  ┌──────────┐  ┌──────────┐
  │  centy   │  │  VS Code  │  │  Web UI  │
  │   CLI    │  │ Extension │  │ (*.centy │
  │          │  │          │  │   .io)   │
  └────┬─────┘  └────┬─────┘  └────┬─────┘
       │              │              │
       │   gRPC/HTTP2 │  gRPC-Web   │
       └──────────────┼──────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         gRPC SERVER  (Tonic + Tokio)                            │
│                         Default: 127.0.0.1:50051                               │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────────────────────────┐    │
│  │  CORS Layer   │→ │  gRPC-Web     │→ │  Logging Layer (request ID)      │    │
│  │ (*.centy.io,  │  │  Adapter      │  │  Structured tracing              │    │
│  │  localhost)   │  │  (HTTP/1.1)   │  │                                  │    │
│  └───────────────┘  └───────────────┘  └──────────────┬───────────────────┘    │
└───────────────────────────────────────────────────────┬────────────────────────-┘
                                                        │
                                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          REQUEST ROUTER  (70+ RPCs)                             │
│                                                                                 │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐ │
│  │  Init   │ │ Issues  │ │  Docs   │ │   PRs   │ │ Generic │ │  Registry   │ │
│  │ Project │ │  CRUD   │ │  CRUD   │ │  CRUD   │ │  Items  │ │  & Config   │ │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └──────┬──────┘ │
│       │           │           │           │           │              │         │
│  ┌────┴────┐ ┌────┴────┐ ┌────┴────┐ ┌────┴─────┐ ┌───┴─────┐ ┌────┴──────┐ │
│  │ Search  │ │  Links  │ │  Users  │ │Workspace │ │  Hooks  │ │ Templates │ │
│  │ & Query │ │ Manage  │ │  Sync   │ │ Manage   │ │ Pre/Post│ │(Handlebars│ │
│  └─────────┘ └─────────┘ └─────────┘ └──────────┘ └─────────┘ └───────────┘ │
└───────────────────────────────────────────────────────┬─────────────────────────┘
                                                        │
                         ┌──────────────────────────────┼──────────────────────┐
                         │           DOMAIN LAYER       │                      │
                         │                              ▼                      │
                         │  ┌────────────────────────────────────────────────┐  │
                         │  │              ENTITY OPERATIONS                 │  │
                         │  │                                                │  │
                         │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐    │  │
                         │  │  │  Issue   │  │   Doc    │  │    PR    │    │  │
                         │  │  │ Module   │  │ Module   │  │  Module  │    │  │
                         │  │  │          │  │          │  │ +git int │    │  │
                         │  │  └────┬─────┘  └────┬─────┘  └────┬─────┘    │  │
                         │  │       └─────────────┼─────────────┘          │  │
                         │  │                     ▼                        │  │
                         │  │  ┌──────────────────────────────────────┐    │  │
                         │  │  │     DAEMON THIN WRAPPERS             │    │  │
                         │  │  │  • CRUD traits    • Lifecycle        │    │  │
                         │  │  │  • Manifest sync  • Asset handling   │    │  │
                         │  │  │  • Error mapping  • Org sync         │    │  │
                         │  │  └──────────────┬───────────────────────┘    │  │
                         │  │                 ▼                            │  │
                         │  │  ┌──────────────────────────────────────┐    │  │
                         │  │  │     mdstore  (crates.io library)     │    │  │
                         │  │  │  • CRUD ops     • Frontmatter engine │    │  │
                         │  │  │  • Validation   • ID strategy        │    │  │
                         │  │  │  • Metadata     • Type config I/O    │    │  │
                         │  │  │  • Reconcile    • Error types        │    │  │
                         │  │  └──────────────────────────────────────┘    │  │
                         │  └────────────────────────────────────────────────┘  │
                         │                              │                      │
                         │  ┌───────────────┐  ┌────────┴────────┐             │
                         │  │ SEARCH ENGINE │  │  HOOK SYSTEM    │             │
                         │  │               │  │                 │             │
                         │  │ Query (PEG)   │  │ pre:issue:create│             │
                         │  │   ↓           │  │ post:doc:update │             │
                         │  │ Parse (Pest)  │  │ pre:pr:delete   │             │
                         │  │   ↓           │  │       ↓         │             │
                         │  │ AST           │  │ Executes bash   │             │
                         │  │   ↓           │  │ with context    │             │
                         │  │ Evaluate      │  │ env vars        │             │
                         │  └───────────────┘  └─────────────────┘             │
                         │                              │                      │
                         │  ┌───────────────┐  ┌────────┴────────┐             │
                         │  │ LINK ENGINE   │  │ RECONCILIATION  │             │
                         │  │               │  │                 │             │
                         │  │ Bidirectional │  │ Integrity check │             │
                         │  │ entity refs   │  │ SHA-256 hashing │             │
                         │  │ (issue↔pr,    │  │ Manifest repair │             │
                         │  │  doc↔issue)   │  │ Schema migrate  │             │
                         │  └───────────────┘  └─────────────────┘             │
                         └─────────────────────────────┬───────────────────────┘
                                                       │
                                                       ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      STORAGE LAYER  (mdstore + File System)                    │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │              mdstore FRONTMATTER ENGINE  (crates.io)                   │    │
│  │      parse_frontmatter<T>() / generate_frontmatter<T>()               │    │
│  │      Reads/Writes YAML frontmatter + Markdown body                    │    │
│  │                                                                         │    │
│  │  ┌────────────────────────────────────┐                                │    │
│  │  │  Example .md file:                 │                                │    │
│  │  │  ---                               │                                │    │
│  │  │  id: 785d290a-...                  │                                │    │
│  │  │  title: Fix login bug              │                                │    │
│  │  │  status: in-progress               │                                │    │
│  │  │  priority: 2                       │                                │    │
│  │  │  created: 2026-02-16T...           │                                │    │
│  │  │  ---                               │                                │    │
│  │  │  # Description                     │                                │    │
│  │  │  The login page crashes when...    │                                │    │
│  │  └────────────────────────────────────┘                                │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                 │
│  PROJECT STORAGE (.centy/)              GLOBAL STORAGE (~/.centy/)              │
│  ┌──────────────────────────┐           ┌──────────────────────────┐            │
│  │  .centy-manifest.json    │           │  projects.json           │            │
│  │  config.json             │           │  (global registry of     │            │
│  │  project.json            │           │   all centy projects)    │            │
│  │  organization.json       │           │                          │            │
│  │  users.json              │           │  workspace-metadata.json │            │
│  │                          │           │  (temp workspace TTLs)   │            │
│  │  issues/                 │           │                          │            │
│  │    ├── {uuid}.md         │           │  logs/                   │            │
│  │    └── {uuid}.md         │           │    └── centy-daemon.log  │            │
│  │  docs/                   │           └──────────────────────────┘            │
│  │    ├── {slug}.md         │                                                   │
│  │    └── {slug}.md         │                                                   │
│  │  prs/                    │                                                   │
│  │    ├── {uuid}.md         │                                                   │
│  │    └── {uuid}.md         │                                                   │
│  │  assets/                 │                                                   │
│  │  templates/              │                                                   │
│  └──────────────────────────┘                                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

  EXTERNAL INTEGRATIONS
  ─────────────────────
  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
  │     Git       │  │    gwq        │  │    Editors    │
  │               │  │  (worktree    │  │               │
  │ • git log     │  │   manager)    │  │ • VS Code     │
  │ • git remote  │  │               │  │   (workspace  │
  │ • user sync   │  │ • Temp        │  │    + tasks)   │
  │ • org infer   │  │   worktrees   │  │ • Terminal    │
  │               │  │ • TTL-based   │  │   (shell)     │
  └───────────────┘  │   cleanup     │  └───────────────┘
                     └───────────────┘
```

## Request Lifecycle

Example: creating an issue.

```
  ┌─────────┐    ┌──────────┐    ┌───────────┐    ┌──────────┐    ┌──────────┐
  │ Client  │───▶│ Validate │───▶│ Pre-Hook  │───▶│ mdstore  │───▶│ Apply    │
  │ Request │    │ Input    │    │ (bash)    │    │ ::create │    │ Template │
  └─────────┘    └──────────┘    └───────────┘    │ (ID gen, │    │(Handlebar│
                                                   │ validate)│    └────┬─────┘
                                                   └──────────┘         │
  ┌─────────┐    ┌──────────┐    ┌───────────┐    ┌──────────┐         │
  │ Return  │◀───│ Org Sync │◀───│ Post-Hook │◀───│ Write .md│◀────────┘
  │Response │    │(if org   │    │ (bash)    │    │ + Update │
  └─────────┘    │ project) │    └───────────┘    │ Manifest │
                 └──────────┘                     └──────────┘
```

## mdstore Library

The core CRUD and storage logic is extracted into the [`mdstore`](https://crates.io/crates/mdstore) crate (published on crates.io), keeping the daemon as a thin orchestration layer.

### What mdstore owns

- **CRUD operations** — `create`, `get`, `list`, `update`, `delete`, `soft_delete`, `restore`, `duplicate`, `move_item`
- **Frontmatter engine** — `parse_frontmatter<T>()` / `generate_frontmatter<T>()` for YAML frontmatter + Markdown body
- **Type configuration** — `TypeConfig`, `TypeFeatures`, `IdStrategy` (UUID vs Slug), `CustomFieldDef`
- **Config I/O** — `config::read_type_config`, `config::write_type_config`, `config::discover_types`
- **Validation** — status validation against allowed values, priority validation against max levels
- **Metadata** — `CommonMetadata`, `Frontmatter` structs with legacy priority migration deserializer
- **ID handling** — `ItemId` enum (UUID/Slug), `Identifiable` trait
- **Reconciliation** — `get_next_display_number`, `reconcile_display_numbers`
- **Error types** — `StoreError` (14+ variants), `ConfigError`, `FrontmatterError`

### What the daemon adds on top

- **gRPC server** — Tonic handlers, proto conversion, error mapping to gRPC status codes
- **Manifest tracking** — `.centy-manifest.json` updates after every CRUD operation
- **Hook system** — pre/post bash hooks on every operation
- **Search engine** — PEG grammar query → AST → evaluator
- **Link engine** — bidirectional entity references
- **Organization sync** — multi-project org-level item syncing
- **Asset management** — file attachments per entity
- **Templates** — Handlebars-based content templates
- **Entity-specific logic** — Issue/Doc/PR modules with custom metadata and behavior

## Multi-Project & Organization

```
  ┌──────────────────────────────────────────────────────────┐
  │  ~/.centy/projects.json  (Global Registry)               │
  │  ┌────────────────────────────────────────────────────┐  │
  │  │  Project A ──┐                                     │  │
  │  │  Project B ──┼── Organization "acme"               │  │
  │  │  Project C ──┘   (inferred from git remote)        │  │
  │  │                                                    │  │
  │  │  Project D ────── Organization "personal"          │  │
  │  │                                                    │  │
  │  │  Org-level issues sync across all member projects  │  │
  │  └────────────────────────────────────────────────────┘  │
  └──────────────────────────────────────────────────────────┘
```

## Key Facts

- **Rust daemon** serving gRPC (Tonic + Tokio) on `127.0.0.1:50051`
- **Completely local-first** — no cloud, no external DB, everything is `.md` files with YAML frontmatter
- **mdstore library** — core CRUD, frontmatter, validation, and config logic extracted to a standalone crate on crates.io
- **Daemon as orchestrator** — thin wrappers around mdstore adding manifest sync, hooks, search, links, org sync, and gRPC serving
- **70+ RPC handlers** for issues, docs, PRs, generic items, search, links, users, workspaces, config
- **Custom query engine** using PEG grammar (Pest) → AST → evaluator
- **Hook system** — pre/post bash hooks on every operation
- **Multi-project** — global registry with organization grouping inferred from git remotes
- **Git-friendly storage** — human-readable markdown, designed to be committed alongside code