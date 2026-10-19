---
name: mcp-usage
description: This skill should be used when the user asks "how do I use centy", "what centy MCP tools are available", "how do I start or stop the daemon", "how do I create issues with centy", or needs a reference for any Centy MCP operation.
allowed-tools: mcp__centy__centy_v1_CentyDaemon_GetDaemonInfo mcp__centy__centy_v1_CentyDaemon_IsInitialized mcp__centy__centy_v1_CentyDaemon_ListItemTypes mcp__centy__centy_v1_CentyDaemon_GetConfig mcp__centy__centy_v1_CentyDaemon_ListProjects
---

# Centy MCP Usage

Reference guide for working with the Centy MCP server from within Claude Code. Use this skill when the user asks how to manage issues, start/stop the daemon, or perform any Centy operation via Claude.

The tables below use the short method name. The full MCP tool name is `centy_v1_CentyDaemon_<Method>` (Claude Code shows it as `mcp__centy__centy_v1_CentyDaemon_<Method>`). Tool arguments use protobuf JSON field names (camelCase).

## Daemon lifecycle

The Centy MCP endpoint is served by the `centy-daemon` process at the local loopback address on port 50052 (path `/mcp`). No tool works while the daemon is down. Start the daemon before any other step.

The endpoint has no start or status tools. If the `centy` server is not connected, check the MCP server list in Claude Code and confirm the daemon is running.

## Core operations

### Projects

| Tool | Purpose |
|------|---------|
| `Init` | Create the project's `.centy/` directory and initial config |
| `IsInitialized` | Check whether a project is initialized; do this before any project-scoped operation |
| `GetConfig` | Read the schema, field definitions and defaults |
| `UpdateConfig` | Update field definitions and other config values |

### Issues

| Tool | Purpose |
|------|---------|
| `CreateItem` | Create an item of a given type |
| `GetItem` | Read one item by its ID or display number |
| `UpdateItem` | Change an item's title, body, status, priority, tags or custom fields |
| `DeleteItem` | Delete an item; hard delete only when forced, otherwise soft delete |
| `ListItems` | List items of a type, optionally narrowed by a filter |
| `SearchItems` | Find an item by its ID across all tracked projects |

> **Important**: the item type for issues is the plural `"issues"`, not `"issue"`. Link types (below) use the singular form.

### Query language

`ListItems` takes its filter as a JSON-encoded query. Filter on fields such as `status` and `priority`. Operators follow the Mongo style: `$eq`, `$in`, `$all`, `$exists`, `$lt`, `$lte`, `$gt` and `$gte`. Example intent: list open issues whose priority is 2 or higher.

### Links

| Tool | Purpose |
|------|---------|
| `CreateLink` | Create a bidirectional relationship between two items |
| `ListLinks` | List all links that touch one item |
| `DeleteLink` | Remove one link by its link ID |

### Registry (multi-project)

| Tool | Purpose |
|------|---------|
| `RegisterProject` | Add a project to the daemon's global registry |
| `UntrackProject` | Remove a project from the registry |
| `ListProjects` | List all registered projects |

### Workspaces

Workspaces are temporary scoped environments used by Claude Code sessions.

| Tool | Purpose |
|------|---------|
| `OpenStandaloneWorkspace` | Open a standalone workspace for the current session, not tied to an issue |
| `ListTempWorkspaces` | List temp workspaces, optionally filtered by source project |
| `CloseTempWorkspace` | Close a temp workspace and clean it up |

## Common workflows

### Create a new issue

1. Confirm the daemon is running.
2. Check that the project is initialized; initialize it first if it is not.
3. Create an item of type `"issues"` with a title and body.
4. Read the new item back by its returned ID to confirm it exists.

### Close an issue

Update the issue's status to closed.

### List open high-priority issues

List items of type `"issues"` with a filter on open status and priority 2 or higher.

### Link two issues

Create a link between the two issues, using the singular item type `"issue"` for both ends. The link type is `"blocks"`.

## Configuration tips

- The `.centy/config.json` file holds item type definitions and field schemas.
- Read the config through `GetConfig` to inspect the current schema before creating custom fields.
- Custom statuses and priorities are defined per-project in `config.json`.

## Getting help

- Run the `install` skill if the daemon is not yet set up.
- Inspect `.centy/` in any initialized project to see the raw Markdown records.
- All records are human-readable `.md` files with YAML frontmatter — safe to read and diff in git.
