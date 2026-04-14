---
# This file is managed by Centy. Use the Centy CLI to modify it.
status: ready
createdAt: 2026-04-14T16:50:31.111880+00:00
updatedAt: 2026-04-14T16:51:18.214812+00:00
deletedAt: 2026-04-14T16:51:18.214812+00:00
persona: developer
---

# MCP server reliability, version tracking, and installation

## User story

As a developer, I want the centy MCP server to install reliably, detect version mismatches with the daemon, and surface actionable errors when something goes wrong, so that my Claude Code integration stays functional without manual debugging.

## Acceptance criteria

- [ ] `centy-mcp` checks its version against the daemon's reported version on startup and emits a clear error when incompatible
- [ ] npx cache corruption (ENOTEMPTY errors) is detected and the user is told how to recover
- [ ] A `CheckForUpdates` gRPC endpoint returns the latest available version string
- [ ] Daemon startup failures include the absolute path to the log file in the error output
- [ ] `cargo install centy-daemon` works as a supported installation path
- [ ] The plugin `.mcp.json` uses the installed binary path rather than `npx -y centy-mcp` when available

## Context

The MCP server and daemon are released independently, causing version drift. npx cache corruption produces ENOTEMPTY errors that break plugin startup. When the daemon itself fails to start, there is no indication of where to look for logs.

## Scope

Covers pending items in epic #423: #401, #383, #185, #153, #355
