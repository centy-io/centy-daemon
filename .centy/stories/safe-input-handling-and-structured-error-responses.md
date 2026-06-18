---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:53:54.368587+00:00
updatedAt: 2026-04-14T16:53:54.368587+00:00
persona: daemon-operator
---

# Safe input handling and structured error responses

## User story

As a daemon operator, I want all inputs properly validated and errors returned as structured JSON with descriptive messages, so that security vulnerabilities are prevented and failures are easy to diagnose.

## Acceptance criteria

- [ ] Shell commands constructed from user input use proper escaping (no injection risk)
- [ ] All gRPC error responses return structured JSON objects with a code, message, and optional details field — not plain strings
- [ ] The project path is validated to be an absolute path before any file operation proceeds
- [ ] Network responses include human-readable error text in addition to the status code
- [ ] Existing e2e tests cover the error response shapes
- [ ] No std::process::exit(1) calls remain in request-handling paths

## Context

The safety work replaced std::process::exit(1) with proper error propagation. Remaining gaps include shell escaping (a security issue), structured JSON errors (needed for client libraries to parse errors reliably), and path validation (prevents path traversal and relative-path bugs).

## Scope

Covers pending items in epic #420: #146, #161, #79, #128
