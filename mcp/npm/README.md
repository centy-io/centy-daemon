# centy-mcp

An MCP (Model Context Protocol) server that exposes the [centy-daemon](https://github.com/centy-io/centy-daemon) gRPC API as tools for AI clients such as Claude Desktop.

Each RPC method in the service becomes an MCP tool, allowing AI agents to manage projects, items, organizations, links, assets, and more.

## Requirements

centy-daemon must be running locally. [Download it here.](https://github.com/centy-io/centy-daemon/releases/latest)

## Usage

### npx (no install required)

```bash
npx centy-mcp
```

### Global install

```bash
npm install -g centy-mcp
centy-mcp
```

By default the MCP server connects to `127.0.0.1:50051`. Override with `CENTY_DAEMON_ADDR`:

```bash
CENTY_DAEMON_ADDR=127.0.0.1:9090 centy-mcp
```

## Claude Code configuration

```bash
claude mcp add centy -s user -- npx -y centy-mcp
```

## Claude Desktop configuration

Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "centy": {
      "command": "npx",
      "args": ["-y", "centy-mcp"]
    }
  }
}
```
