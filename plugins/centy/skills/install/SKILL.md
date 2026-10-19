---
name: install
description: This skill should be used when the user asks to "install centy-daemon", "set up centy", "clone and build centy-daemon", "wire up the centy MCP server", or needs help getting centy-daemon running for the first time.
version: 1.0.0
---

# Install centy-daemon

Guide the user step-by-step through setting up `centy-daemon` locally and wiring it up as an MCP server inside Claude Code. Work through each step in order, confirming success before moving on. If a step fails, diagnose and fix before continuing.

## Prerequisites check

Before starting, verify the user has the required tools: a Rust toolchain at version 1.85 or newer, the cargo build tool, and git.

If Rust is missing, direct the user to the rustup site and wait for confirmation.

## Step 1 — Clone the repository

Clone the `centy-io/centy-daemon` repository from GitHub and enter its directory.

## Step 2 — Build the daemon binary

Build the daemon in release mode. The binary lands in the release output folder of the build target. The first build takes a few minutes.

## Step 3 — Install the binary (optional but recommended)

Copy the release binary to a directory on the user's `PATH`, such as a per-user bin folder or a system-wide bin folder (the latter may need elevated rights on macOS or Linux).

Alternatively, set the `CENTY_DAEMON_PATH` environment variable to the absolute path of the binary and skip this step.

## Step 4 — Start the daemon

Start the daemon. By default it listens for gRPC on the loopback address on port 50051 and runs in the foreground. For a persistent setup, run it under a process manager that starts it on login. For a quick test, run it in a separate terminal.

Verify it is running by listing the gRPC services on the loopback address. The output should name the `centy.CentyDaemon` service.

## Step 5 — Register the MCP server in Claude Code

The daemon serves MCP over HTTP on the loopback address at port 50052, path `/mcp`. Claude Code connects to that URL; do not spawn a process for it. Make sure the daemon is running first.

Register a server named `centy` with HTTP transport and that URL. Do this in the project's MCP configuration file, or through the Claude Code MCP add command for global access.

If the daemon uses a different MCP port, start it with that address set through its MCP address flag or the `CENTY_MCP_ADDR` environment variable, then use the matching URL. The address must be loopback.

## Step 6 — Verify the integration

Restart Claude Code, or reload the MCP servers, then confirm the tools are available:

- Open the MCP status view in Claude Code and check that the `centy` server is listed.
- Ask Claude to call the `GetDaemonInfo` tool (full name `centy_v1_CentyDaemon_GetDaemonInfo`); it should return the daemon version.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| MCP connection refused on `127.0.0.1:50052` | Make sure the daemon is running |
| Wrong MCP port | Use the URL matching the daemon's MCP address setting |
| Rust toolchain too old | Update the stable toolchain through rustup |

Once complete, tell the user they are ready to use Centy. Suggest running the `mcp-usage` skill to learn about available tools.
