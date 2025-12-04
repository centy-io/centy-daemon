# centy-daemon

A gRPC daemon service for [Centy](https://github.com/centy-io/centy-cli) - a local-first issue and documentation tracker.

## Overview

centy-daemon manages `.centy` folder operations, providing a backend service for:

- Initializing and reconciling `.centy` project folders
- Creating and managing issues with metadata
- Tracking managed files with SHA-256 integrity hashes
- Configuration management for custom fields and defaults

## Requirements

- Rust 1.70+ (2021 edition)
- Protocol Buffers compiler (`protoc`)

## Installation

```bash
git clone https://github.com/centy-io/centy-daemon.git
cd centy-daemon
cargo build --release
```

## Usage

### Start the daemon

```bash
# Default address: 127.0.0.1:50051
cargo run --release

# Custom address
CENTY_DAEMON_ADDR=127.0.0.1:50052 cargo run --release
```

### gRPC API

The daemon supports both **native gRPC** (HTTP/2) and **gRPC-Web** (HTTP/1.1), making it compatible with:
- Native gRPC clients (CLI tools, backend services)
- Browser-based applications (via gRPC-Web/Connect)

CORS is enabled for localhost origins in development mode.

The daemon exposes the `CentyDaemon` service with the following RPCs:

| RPC | Description |
|-----|-------------|
| `Init` | Initialize a `.centy` folder in a project directory |
| `GetReconciliationPlan` | Preview changes without executing |
| `ExecuteReconciliation` | Apply reconciliation with user decisions |
| `CreateIssue` | Create a new issue with title, description, and metadata |
| `GetNextIssueNumber` | Get the next sequential issue number |
| `GetManifest` | Read the project manifest |
| `GetConfig` | Read project configuration |
| `IsInitialized` | Check if centy is initialized in a directory |

See [`proto/centy.proto`](proto/centy.proto) for the full API specification.

### Testing with grpcui

[grpcui](https://github.com/fullstorydev/grpcui) provides a web-based UI for interacting with the gRPC API.

```bash
# Install grpcui
go install github.com/fullstorydev/grpcui/cmd/grpcui@latest

# Start the daemon first, then launch grpcui
grpcui -plaintext 127.0.0.1:50051
```

This opens a browser with an interactive interface to call any RPC method.

## Project Structure

```
.centy/                     # Created in your project root
├── .centy-manifest.json    # Tracks managed files with hashes
├── config.json             # Custom fields and defaults
├── README.md               # Project README
├── issues/                 # Issue storage
│   └── 0001/
│       ├── issue.md        # Issue content
│       ├── metadata.json   # Status, priority, timestamps
│       └── assets/         # Attachments
├── docs/                   # Documentation
└── assets/                 # Shared assets
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, and contribution guidelines.

## License

MIT
