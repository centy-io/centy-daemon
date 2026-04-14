---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:53:44.467461+00:00
updatedAt: 2026-04-14T16:53:44.467461+00:00
persona: developer
---

# Legacy proto types and deprecated daemon concepts removed

## User story

As a developer, I want legacy proto types and deprecated API concepts removed from the daemon, so that the wire protocol and daemon internals stay clean and free of dead code that confuses new contributors.

## Acceptance criteria

- [ ] The PR entity type is removed from proto definitions and all associated Rust handling code
- [ ] Agent and LLM logic is fully removed from the daemon (no dead code remains)
- [ ] The legacy allowedStates field is removed from both CentyConfig and the gRPC API
- [ ] All e2e tests pass after each removal
- [ ] Proto package remains at centy.v1 with no version-incompatible changes introduced

## Context

The proto cleanup epic already versioned the API (centy.v1) and removed the EntityType enum. The remaining items (PR type, agent/LLM logic, allowedStates) are unused code paths that create confusion and inflate binary size.

## Scope

Covers pending items in epic #421: #368, #369, #202
