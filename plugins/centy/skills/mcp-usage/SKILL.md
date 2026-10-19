---
name: mcp-usage
description: This skill should be used when the user asks "how do I use centy", "what centy MCP tools are available", "how do I start or stop the daemon", "how do I create issues with centy", or needs a reference for any Centy MCP operation.
allowed-tools: mcp__centy__centy_v1_CentyDaemon_GetDaemonInfo mcp__centy__centy_v1_CentyDaemon_IsInitialized mcp__centy__centy_v1_CentyDaemon_ListItemTypes mcp__centy__centy_v1_CentyDaemon_GetConfig mcp__centy__centy_v1_CentyDaemon_ListProjects
---

# Centy MCP Usage

Reference guide for working with the Centy MCP server from within Claude Code. Use this skill when the user asks how to manage issues, start/stop the daemon, or perform any Centy operation via Claude.

The tables below use the short method name. The full MCP tool name is `centy_v1_CentyDaemon_<Method>` (Claude Code shows it as `mcp__centy__centy_v1_CentyDaemon_<Method>`). Arguments use protobuf JSON field names (camelCase), for example `projectPath` and `itemType`.

## Daemon lifecycle

The Centy MCP endpoint is served by `centy-daemon` at `http://127.0.0.1:50052/mcp`. No tool works while the daemon is down. Start the daemon first:

```bash
centy-daemon &
```

The endpoint has no start or status tools. If the `centy` server is not connected, check `/mcp` in Claude Code and confirm the daemon is running.

## Core operations

### Projects

| Tool | Key inputs | Notes |
|------|-----------|-------|
| `Init` | `projectPath` | Creates the `.centy/` directory and initial config |
| `IsInitialized` | `projectPath` | Check before any project-scoped operation |
| `GetConfig` | `projectPath` | Read schema, field definitions, defaults |
| `UpdateConfig` | `projectPath`, `config` | Update field definitions |

### Issues

| Tool | Key inputs |
|------|-----------|
| `CreateItem` (`itemType` `issues`) | `projectPath`, `title`, optional `status`, `priority`, `body` |
| `GetItem` (`itemType` `issues`) | `projectPath`, `itemId` |
| `UpdateItem` (`itemType` `issues`) | `projectPath`, `itemId`, fields to change |
| `DeleteItem` (`itemType` `issues`) | `projectPath`, `itemId`, optional `force` |
| `ListItems` (`itemType` `issues`) | `projectPath`, optional `filter` |
| `SearchItems` (`itemType` `issues`) | `itemId` (finds the item across all tracked projects) |

> **Important**: pass `itemType` as `"issues"` (plural), not `"issue"`.

### Query language

Pass a JSON-encoded query in the `filter` field of `ListItems`. Examples:

```
{"status": "open"}
{"status": {"$in": ["open", "in-progress"]}}
{"status": "open", "priority": {"$gte": 2}}
```

Supported operators include `$eq`, `$in`, `$all`, `$exists`, `$lt`, `$lte`, `$gt` and `$gte`.

### Links

| Tool | Description |
|------|-------------|
| `CreateLink` | Create a bidirectional relationship between two items |
| `ListLinks` | List all links for an item (`entityId`, `entityItemType`) |
| `DeleteLink` | Remove a specific link (`linkId`) |

### Registry (multi-project)

| Tool | Description |
|------|-------------|
| `RegisterProject` | Add a project to the daemon's global registry |
| `UntrackProject` | Remove a project from the registry |
| `ListProjects` | List all registered projects |

### Workspaces

Workspaces are temporary scoped environments used by Claude Code sessions.

| Tool | Description |
|------|-------------|
| `OpenStandaloneWorkspace` | Open a standalone workspace for the current session (not tied to an issue) |
| `ListTempWorkspaces` | List temp workspaces, optionally filtered by `sourceProjectPath` |
| `CloseTempWorkspace` | Close and clean up a temp workspace (`workspacePath`) |

## Common workflows

### Create a new issue

```
1. Confirm `centy-daemon` is running (`centy-daemon &` if not)
2. IsInitialized { projectPath } (run Init { projectPath } if needed)
3. CreateItem { itemType: "issues", projectPath, title: "...", body: "..." }
4. GetItem { itemType: "issues", projectPath, itemId: <returned id> }
```

### Close an issue

```
UpdateItem { itemType: "issues", projectPath, itemId, status: "closed" }
```

### List open high-priority issues

```
ListItems { itemType: "issues", projectPath, filter: "{\"status\": \"open\", \"priority\": {\"$gte\": 2}}" }
```

### Link two issues

```
CreateLink { projectPath, sourceId: "abc", targetId: "xyz", linkType: "blocks", sourceItemType: "issue", targetItemType: "issue" }
```

## Configuration tips

- The `.centy/config.json` file holds item type definitions and field schemas.
- Use `GetConfig` to inspect the current schema before creating custom fields.
- Custom statuses and priorities are defined per-project in `config.json`.

## Getting help

- Run the `install` skill if the daemon is not yet set up.
- Inspect `.centy/` in any initialized project to see the raw Markdown records.
- All records are human-readable `.md` files with YAML frontmatter — safe to read and diff in git.
