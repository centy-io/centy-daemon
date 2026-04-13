# Centy MCP Usage

Reference guide for working with the Centy MCP server from within Claude Code. Use this skill when the user asks how to manage issues, start/stop the daemon, or perform any Centy operation via Claude.

## Daemon lifecycle

The MCP server (`centy-mcp`) exposes two lifecycle tools that do not require the daemon to already be running:

| Tool | Description |
|------|-------------|
| `IsRunning` | Returns whether `centy-daemon` is currently listening on the configured address |
| `StartDaemon` | Spawns the daemon as a background process if it is not already running |

**Typical pattern** — always call `IsRunning` first, then `StartDaemon` if needed, before calling any other tool:

```
IsRunning → (if not running) StartDaemon → <desired tool>
```

## Core operations

### Projects

| Tool | Key inputs | Notes |
|------|-----------|-------|
| `InitializeProject` | `project_path` | Creates the `.centy/` directory and initial config |
| `IsInitialized` | `project_path` | Check before any project-scoped operation |
| `GetProjectConfig` | `project_path` | Read schema, field definitions, defaults |
| `UpdateProjectConfig` | `project_path`, config fields | Update field definitions |

### Issues

| Tool | Key inputs |
|------|-----------|
| `CreateItem` (type `issues`) | `project_path`, `title`, optional `status`, `priority`, `body` |
| `GetItem` (type `issues`) | `project_path`, `item_id` |
| `UpdateItem` (type `issues`) | `project_path`, `item_id`, fields to change |
| `DeleteItem` (type `issues`) | `project_path`, `item_id` |
| `ListItems` (type `issues`) | `project_path`, optional `query` |
| `SearchItems` (type `issues`) | `project_path`, `query` |

> **Important**: pass `item_type` as `"issues"` (plural), not `"issue"`.

### Query language

Use the `query` parameter on `ListItems` / `SearchItems` with Centy's query syntax:

```
status = "open"
status = "open" AND priority >= 2
title ~ "auth*"          # wildcard
body ~ /grpc/i           # regex
status != "closed"
```

### Links

| Tool | Description |
|------|-------------|
| `CreateLink` | Create a bidirectional relationship between two items |
| `GetLinks` | List all links for an item |
| `DeleteLink` | Remove a specific link |

### Registry (multi-project)

| Tool | Description |
|------|-------------|
| `RegisterProject` | Add a project to the daemon's global registry |
| `UnregisterProject` | Remove a project from the registry |
| `ListProjects` | List all registered projects |

### Workspaces

Workspaces are temporary scoped environments used by Claude Code sessions.

| Tool | Description |
|------|-------------|
| `CreateWorkspace` | Open a workspace for the current session |
| `GetWorkspace` | Retrieve an existing workspace |
| `DeleteWorkspace` | Close and clean up a workspace |

## Common workflows

### Create a new issue

```
1. IsRunning (start daemon if needed)
2. IsInitialized { project_path } (init if needed)
3. CreateItem { item_type: "issues", project_path, title: "...", body: "..." }
4. GetItem { item_type: "issues", project_path, item_id: <returned id> }
```

### Close an issue

```
UpdateItem { item_type: "issues", project_path, item_id, status: "closed" }
```

### List open high-priority issues

```
ListItems { item_type: "issues", project_path, query: "status = \"open\" AND priority >= 2" }
```

### Link two issues

```
CreateLink { project_path, source_id: "abc", target_id: "xyz", link_type: "blocks" }
```

## Configuration tips

- The `.centy/config.json` file holds item type definitions and field schemas.
- Use `GetProjectConfig` to inspect the current schema before creating custom fields.
- Custom statuses and priorities are defined per-project in `config.json`.

## Getting help

- Run the `install` skill if the daemon is not yet set up.
- Inspect `.centy/` in any initialized project to see the raw Markdown records.
- All records are human-readable `.md` files with YAML frontmatter — safe to read and diff in git.
