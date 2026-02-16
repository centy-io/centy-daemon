---
title: "Daemon Architecture Overview"
createdAt: "2026-02-16T15:04:56.752536+00:00"
updatedAt: "2026-02-16T15:06:54.788323+00:00"
---

# Daemon Architecture Overview

# Daemon Architecture Overview

ASCII diagram covering the full architecture of centy-daemon: clients, gRPC server, domain layer, storage, and integrations.

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
                         │  │  │        SHARED CORE ABSTRACTIONS      │    │  │
                         │  │  │  • CRUD traits    • Validation       │    │  │
                         │  │  │  • Lifecycle       • Metadata        │    │  │
                         │  │  │  • Move/Duplicate  • Error types     │    │  │
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
│                           STORAGE LAYER (File System)                           │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                     FRONTMATTER ENGINE                                  │    │
│  │         Reads/Writes YAML frontmatter + Markdown body                  │    │
│  │                                                                         │    │
│  │  ┌────────────────────────────────────┐                                │    │
│  │  │  Example .md file:                 │                                │    │
│  │  │  ---                               │                                │    │
│  │  │  id: 785d290a-...                  │                                │    │
│  │  │  title: Fix login bug              │                                │    │
│  │  │  status: in-progress               │                                │    │
│  │  │  priority: high                    │                                │    │
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
  │ Client  │───▶│ Validate │───▶│ Pre-Hook  │───▶│ Generate │───▶│ Apply    │
  │ Request │    │ Input    │    │ (bash)    │    │ UUID +   │    │ Template │
  └─────────┘    └──────────┘    └───────────┘    │ Display# │    │(Handlebar│
                                                   └──────────┘    └────┬─────┘
                                                                        │
  ┌─────────┐    ┌──────────┐    ┌───────────┐    ┌──────────┐         │
  │ Return  │◀───│ Org Sync │◀───│ Post-Hook │◀───│ Write .md│◀────────┘
  │Response │    │(if org   │    │ (bash)    │    │ + Update │
  └─────────┘    │ project) │    └───────────┘    │ Manifest │
                 └──────────┘                     └──────────┘
```

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
- **70+ RPC handlers** for issues, docs, PRs, generic items, search, links, users, workspaces, config
- **Custom query engine** using PEG grammar (Pest) → AST → evaluator
- **Hook system** — pre/post bash hooks on every operation
- **Multi-project** — global registry with organization grouping inferred from git remotes
- **Git-friendly storage** — human-readable markdown, designed to be committed alongside code