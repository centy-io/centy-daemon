---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:54:43.276340+00:00
updatedAt: 2026-04-14T16:54:43.276340+00:00
persona: developer
---

# Clean hooks and config architecture with validated UpdateConfig

## User story

As a developer, I want centy's configuration to be updatable through a validated gRPC API and lifecycle hooks to support HTTP webhooks and gRPC subscriptions, so that I can automate project workflows reliably without editing YAML by hand or knowing daemon internals.

## Acceptance criteria

- [ ] An UpdateConfig RPC exists that accepts dot-key paths, validates values against the schema, and flattens updates atomically
- [ ] The legacy allowedStates field is removed from CentyConfig and the gRPC API
- [ ] Lifecycle hooks support HTTP webhook delivery (POST with JSON payload) in addition to command execution
- [ ] Lifecycle hooks support gRPC subscriptions so external services can receive events as a stream
- [ ] Hook execution history is stored and queryable (last N events per hook)
- [ ] End-to-end tests cover hook delivery for all supported transports
- [ ] Config validation middleware rejects invalid values before they are written to disk

## Context

The hooks migration (config.json to hooks.yaml) and ConfigService extraction are complete. The remaining work is the UpdateConfig API (so config can be changed without editing JSON by hand), removing the last legacy field (allowedStates), and extending hooks beyond command execution to HTTP and gRPC transports.

## Scope

Covers pending items in epic #415: #373, #202, #372
