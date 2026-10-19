---
name: install
description: This skill should be used when the user asks to "install centy-daemon", "set up centy", "clone and build centy-daemon", "wire up the centy MCP server", or needs help getting centy-daemon running for the first time.
version: 1.0.0
---

# Install centy-daemon

Guide the user step-by-step through setting up `centy-daemon` locally and wiring it up as an MCP server inside Claude Code. Work through each step in order, confirming success before moving on. If a step fails, diagnose and fix before continuing.

## Prerequisites check

Before starting, verify the user has the required tools:

```bash
rustc --version   # needs 1.85+
cargo --version
git --version
```

If Rust is missing, direct them to https://rustup.rs and wait for confirmation.

## Step 1 — Clone the repository

```bash
git clone https://github.com/centy-io/centy-daemon.git
cd centy-daemon
```

## Step 2 — Build the daemon binary

```bash
cargo build --release
```

This places the binary at `target/release/centy-daemon`. The build takes a few minutes on first run.

## Step 3 — Install the binary (optional but recommended)

Make the daemon available system-wide by copying it to a directory on `$PATH`:

```bash
# macOS / Linux
cp target/release/centy-daemon ~/.local/bin/centy-daemon
# or
sudo cp target/release/centy-daemon /usr/local/bin/centy-daemon
```

Alternatively, set `CENTY_DAEMON_PATH` to the absolute path of the binary and skip this step.

## Step 4 — Start the daemon

```bash
# Runs in the foreground on 127.0.0.1:50051
centy-daemon

# Run as a background process on login (launchd / systemd / any process manager)
# For a quick test, just run it in a separate terminal tab
```

Verify it is running:

```bash
grpcurl -plaintext 127.0.0.1:50051 list   # should print centy.CentyDaemon
```

## Step 5 — Register the MCP server in Claude Code

The daemon serves MCP over HTTP at `http://127.0.0.1:50052/mcp`. Claude Code connects to that URL. Do not spawn a process for it. Make sure `centy-daemon` is running first.

Add the following block to the project's `.mcp.json` (or run the command for global access):

```json
{
  "mcpServers": {
    "centy": {
      "type": "http",
      "url": "http://127.0.0.1:50052/mcp"
    }
  }
}
```

```bash
claude mcp add --transport http centy http://127.0.0.1:50052/mcp
```

If the daemon uses a different MCP port, start it with `centy-daemon --mcp-addr 127.0.0.1:PORT` (or set `CENTY_MCP_ADDR`), then use that URL. The address must be loopback.

## Step 6 — Verify the integration

Restart Claude Code (or reload the MCP servers) then confirm the tools are available:

- Run `/mcp` in Claude Code — you should see the `centy` server listed.
- Ask Claude to call `centy_v1_CentyDaemon_GetDaemonInfo` — it should return the daemon version.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| MCP connection refused on `127.0.0.1:50052` | Make sure `centy-daemon` is running (`centy-daemon &`) |
| Wrong MCP port | Use the URL matching `--mcp-addr` / `CENTY_MCP_ADDR` on the daemon |
| `rustc --version` too old | Run `rustup update stable` |

Once complete, tell the user they are ready to use Centy. Suggest running the `mcp-usage` skill to learn about available tools.
