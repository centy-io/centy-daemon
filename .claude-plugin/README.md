# .claude-plugin

Plugin marketplace manifest for centy-daemon. Claude Code reads `marketplace.json` here to discover the plugins this repo exposes, each of which points to a directory containing its own `plugin.json` and skills.

## Using the plugin

### Install from another repo

Add this repo as a marketplace, then install the plugin:

```
/plugin marketplace add centy-io/centy-daemon
/plugin install centy@centy-daemon
```

### Invoke skills

Once installed, invoke any skill with the `centy:` prefix:

```
/centy:install
/centy:mcp-usage
/centy:compact
```

## Adding a plugin

1. Create `plugins/<name>/` with a `.claude-plugin/plugin.json` inside it.
2. Register it in `marketplace.json`.

## Adding a skill to an existing plugin

1. Create `plugins/centy/skills/<skill-name>/SKILL.md`.
2. Add the skill entry to `plugins/centy/.claude-plugin/plugin.json`.
