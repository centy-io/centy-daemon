# centy-daemon MCP Server

An MCP (Model Context Protocol) server that exposes the `CentyDaemon` gRPC API as tools for AI clients such as Claude Desktop.

Each RPC method in the service becomes an MCP tool, allowing AI agents to manage projects, items, organizations, links, assets, and more.

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

The daemon must already be running. By default the MCP server connects to `127.0.0.1:50051`. Override with `CENTY_DAEMON_ADDR`:

```bash
CENTY_DAEMON_ADDR=127.0.0.1:9090 centy-mcp
```

## Claude Desktop configuration

Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "centy": {
      "command": "npx",
      "args": ["centy-mcp"]
    }
  }
}
```

## How it works

Code generation is driven by [`protoc-gen-go-mcp`](https://github.com/redpanda-data/protoc-gen-go-mcp). Running `make generate` from the repo root compiles the proto definitions in `proto/` and writes the generated Go stubs to `mcp/gen/` (gitignored). The `main.go` entrypoint wires the generated handlers to a ConnectRPC client that forwards calls to the running daemon over gRPC.

```
proto/centy/v1/*.proto
        │
        ▼ buf + protoc-gen-go-mcp
mcp/gen/centy/v1/
  ├── *.pb.go              (proto messages)
  ├── centyv1connect/      (ConnectRPC client stubs)
  └── centyv1mcp/          (MCP tool handlers)
        │
        ▼ main.go
MCP server (stdio) ──► centy-daemon gRPC (127.0.0.1:50051)
```

The npm package (`mcp/npm/`) downloads the correct pre-built binary for the current platform from GitHub Releases on install.

## Development

### Prerequisites

- [buf](https://buf.build/docs/installation) — `brew install bufbuild/buf/buf`
- Go 1.26+

### Generate and build

```bash
# From the repo root
make generate

# Build the binary
cd mcp && go build -o centy-mcp .
```
