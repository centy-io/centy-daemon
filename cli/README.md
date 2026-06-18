# centy-cli

A command-line client for [centy-daemon](../README.md), auto-generated from the gRPC proto definitions.

## Overview

`centy-cli` exposes all 80+ `CentyDaemon` RPCs as subcommands, letting you interact with a running daemon directly from the terminal — no code required.

Commands are generated from [`proto/centy.proto`](../proto/centy.proto) via [`protoc-gen-cobra`](https://github.com/NathanBaulch/protoc-gen-cobra), so the CLI always stays in sync with the API.

## Installation

```bash
cd cli
make build-cli
# produces ./centy
```

Or install directly with Go:

```bash
go install github.com/centy-io/centy-daemon/centy@latest
```

## Usage

```bash
# Default: connects to 127.0.0.1:50051
centy centy-daemon <rpc> [flags]

# Custom server address
CENTY_DAEMON_ADDR=127.0.0.1:50052 centy centy-daemon <rpc> [flags]

# TLS connection
centy --tls --tls-ca-cert-file ca.crt centy-daemon <rpc> [flags]
```

### Examples

```bash
# Check if a project is initialized
centy centy-daemon is-initialized --project-path /path/to/project

# List all items in a project
centy centy-daemon list-items --project-path /path/to/project

# Pass request data as JSON
centy centy-daemon create-item --json '{"project_path":"/path/to/project","item_type":"issues","title":"My issue"}'
```

Run `centy centy-daemon --help` to see all available subcommands.

## Regenerating the CLI

When the proto definitions change, regenerate the CLI code:

```bash
make generate
```

This runs `buf generate --template buf.gen.yaml ../proto` and updates everything under `gen/`.

## Configuration

| Environment variable  | Default            | Description                |
|-----------------------|--------------------|----------------------------|
| `CENTY_DAEMON_ADDR`   | `127.0.0.1:50051`  | gRPC server address        |

Global flags (available on every subcommand):

| Flag                  | Description                              |
|-----------------------|------------------------------------------|
| `--tls`               | Enable TLS                               |
| `--tls-ca-cert-file`  | Path to CA certificate                   |
| `--tls-cert-file`     | Path to client certificate               |
| `--tls-key-file`      | Path to client key                       |

## License

MIT
