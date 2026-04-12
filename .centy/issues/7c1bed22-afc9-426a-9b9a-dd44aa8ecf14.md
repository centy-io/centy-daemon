---
# This file is managed by Centy. Use the Centy CLI to modify it.
displayNumber: 403
status: closed
priority: 2
createdAt: 2026-04-12T20:17:31.203612+00:00
updatedAt: 2026-04-12T20:31:17.156781+00:00
---

# MCP server should emit a clear error when its version is incompatible with the daemon

## Problem

When `centy-mcp` is out of date relative to the running `centy-daemon`, the connection fails silently. Claude Code just shows "Failed to reconnect to centy." with no indication of why.

## Goal

On startup, `centy-mcp` should detect a version mismatch with the daemon and emit a human-readable error before failing, e.g.:

```
centy-mcp v0.9.2 is incompatible with centy-daemon v0.10.5. Please update centy-mcp.
```

## Implementation notes

- The daemon already exposes a `GetDaemonInfo` RPC that returns version info — the MCP server can call this on startup
- Define a compatibility policy (semver minor? exact match?) and encode it in the check
- The error should surface in a way MCP clients (e.g. Claude Code) can display to the user
