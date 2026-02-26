---
createdAt: 2026-02-26T00:42:12.888841+00:00
updatedAt: 2026-02-26T00:42:12.888841+00:00
customFields:
  acceptance-criteria: User can install a plugin with one command. Daemon calls the plugin on declared events. Plugin health is visible in centy plugin list. A crashing plugin is marked degraded and does not block the daemon. Uninstalling a plugin removes its hook subscriptions.
  persona: jordan-plugin-author
---

# Install a plugin and have the daemon run it on lifecycle events

## Context

Jordan built a GitHub sync plugin — a binary that reads centy hook payloads and mirrors items to GitHub Issues. Right now Jordan's users must manually add a hooks entry in `.centy/hooks.yaml`, point it at the correct binary path, and figure out which events to subscribe to. There is no feedback if the plugin crashes or is misconfigured.

## The Job To Be Done

> "I built a centy-github-sync plugin. I want users to install it with one command and have the daemon load it, track its version, and call it on the right events — without anyone touching YAML by hand."

## Concrete Scenarios

- **First install** — user runs `centy plugin install centy-github-sync`. Daemon discovers the plugin manifest, registers it, wires up the declared hook subscriptions, and reports a healthy status.
- **Plugin crash** — plugin process exits non-zero on a hook call. Daemon marks it degraded, surfaces the error in `centy plugin list`, and stops calling it until the user re-enables or upgrades.
- **Version upgrade** — user runs `centy plugin upgrade centy-github-sync`. Daemon swaps the binary, re-validates the manifest, and restarts the subscriptions without losing existing config.
- **Plugin introspection** — user runs `centy plugin list` and sees: name, version, status (healthy / degraded / disabled), subscribed events.

## What Centy Could Offer

- Plugin manifest format (`centy-plugin.toml`) declaring name, version, binary path, and hook subscriptions
- `centy plugin install / uninstall / list / enable / disable` commands
- Daemon-side plugin registry — stores installed plugins, their config, and last-known health
- Health tracking: last exit code, last run time, consecutive failure count
- Hook executor routes events to both native hooks and installed plugins
- Plugin sandbox: run plugin binary as subprocess with timeout and stdio capture
