---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:54:32.015424+00:00
updatedAt: 2026-04-14T16:54:32.015424+00:00
persona: project-manager
---

# Config-driven custom item types usable end to end

## User story

As a project manager, I want to define custom item types via a config.yaml file without touching daemon code, so that centy adapts to my team's workflow — bugs, tasks, decisions, or anything else — using the same generic storage and gRPC API as built-in types.

## Acceptance criteria

- [ ] A config.yaml schema for item types is defined and validated by the daemon at startup
- [ ] The daemon builds an item type registry by scanning .centy/*/config.yaml on startup
- [ ] centy init generates default config.yaml files for issues and docs
- [ ] A one-time migration inserts config.yaml into existing projects without data loss
- [ ] The hardcoded ItemType enum is replaced by dynamic registry lookup throughout the daemon
- [ ] A generic storage layer handles all item types (built-in and custom) through one code path
- [ ] The gRPC ItemType enum is replaced by a string item_type field in all RPCs
- [ ] Generic CRUD RPCs (CreateItem, ListItems, UpdateItem, DeleteItem, SoftDeleteItem, RestoreItem) handle all item types
- [ ] Old per-entity gRPC RPCs are removed after the generic ones are in place
- [ ] Field values are validated against the config.yaml schema on write
- [ ] The softDelete feature flag is removed — soft delete is always enabled
- [ ] A CreateItemType gRPC endpoint allows dynamic item type creation at runtime
- [ ] Delete, move, and duplicate logic is unified for all item types
- [ ] Hardcoded doc entity code is removed and routed through the generic layer
- [ ] Custom item types with templates work end to end from CLI to storage

## Context

The generic item type system is the largest architectural initiative in the codebase. Built-in types (issue, doc) are currently handled by special-cased code; the goal is to route everything through a single generic path driven by config.yaml, making custom types a first-class feature.

## Scope

Covers pending items in epic #1: #171, #172, #173, #174, #176, #177, #178, #190, #191, #180, #189, #193, #194/#195/#196, #211, #130
