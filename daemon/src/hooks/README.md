# Hooks

The hook system lets you run shell commands before or after item operations (create, update, delete, etc.). Hooks are configured per-project in `.centy/hooks.yaml`.

## Configuration

```yaml
hooks:
  - pattern: "issue.creating"
    command: "validate-issue.sh"
    timeout: 30    # seconds, default: 30
    async: false   # default: false
    enabled: true  # default: true
```

Each hook has:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `pattern` | string | required | `<item_type>.<event>` — supports `*` wildcards |
| `command` | string | required | Shell command executed via `bash -c` |
| `timeout` | integer | `30` | Seconds before the hook process is killed |
| `async` | bool | `false` | If true, the hook is spawned in the background (post-hooks only) |
| `enabled` | bool | `true` | Set to false to disable without removing |

## Patterns

Format: `<item_type>.<event>`

The item type can be any string (`issue`, `doc`, a custom type, or `*`). The event must be one of the valid event names below, or `*`.

| Operation | Pre event | Post event |
|-----------|-----------|------------|
| create | `creating` | `created` |
| update | `updating` | `updated` |
| delete | `deleting` | `deleted` |
| soft-delete | `soft-deleting` | `soft-deleted` |
| restore | `restoring` | `restored` |
| move | `moving` | `moved` |
| duplicate | `duplicating` | `duplicated` |

Examples:

```
issue.creating    # before creating an issue
*.created         # after creating any item type
doc.deleting      # before deleting a doc
*.*               # matches everything
```

When multiple hooks match, they run in **specificity order** — most specific first:

| Pattern | Specificity |
|---------|-------------|
| `issue.creating` | 2 (both segments exact) |
| `issue.*` or `*.creating` | 1 (one segment exact) |
| `*.*` | 0 (both wildcards) |

## Hook context

Each hook receives context two ways:

**Environment variables:**

| Variable | Description |
|----------|-------------|
| `CENTY_PHASE` | `pre` or `post` |
| `CENTY_ITEM_TYPE` | e.g. `issue`, `doc` |
| `CENTY_OPERATION` | e.g. `create`, `update` |
| `CENTY_PROJECT_PATH` | Absolute path to the project root |
| `CENTY_ITEM_ID` | Item ID (not set for pre-create hooks) |

**Stdin:** Full context as JSON, including `request_data` (the operation payload) and `success` (post-hooks only). Read it with `cat` or pipe to `jq`.

```bash
#!/usr/bin/env bash
context=$(cat)
title=$(echo "$context" | jq -r '.request_data.title // empty')
```

The working directory for all hook commands is `.centy/` within the project.

## Pre-hooks vs post-hooks

**Pre-hooks** (`creating`, `updating`, `deleting`, ...):
- Run synchronously before the operation executes.
- A **non-zero exit code aborts the operation** and returns an error to the caller.
- Use these for validation and policy enforcement.

**Post-hooks** (`created`, `updated`, `deleted`, ...):
- Run after the operation completes (regardless of success/failure).
- Exit code is ignored; failures are logged as warnings.
- If `async: true`, the hook is spawned in the background and does not block the response.
- Use these for notifications, side-effects, and integrations.

## Examples

Validate that an issue has a non-empty title before creation:

```yaml
hooks:
  - pattern: "issue.creating"
    command: |
      title=$(cat | jq -r '.request_data.title // empty')
      if [ -z "$title" ]; then
        echo "Title is required" >&2
        exit 1
      fi
```

Send a notification after any item is created (non-blocking):

```yaml
hooks:
  - pattern: "*.created"
    command: "notify.sh"
    async: true
```
