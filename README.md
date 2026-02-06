# centy-daemon

A file-based database engine that stores structured data as Markdown files with YAML frontmatter, exposed via gRPC.

## What is centy-daemon?

centy-daemon is the **storage and query engine** behind [Centy](https://centy.io). It persists all data directly to the filesystem — no external database required. Every record is a human-readable Markdown file with structured metadata in YAML frontmatter, stored inside a `.centy` directory that can be version-controlled with git.

The daemon runs as a local gRPC service and provides:

- **File-based persistence** — all data lives as `.md` files on disk
- **Structured metadata** — YAML frontmatter for typed fields (status, priority, timestamps, custom fields)
- **CRUD operations** — create, read, update, delete for all entity types
- **Query engine** — advanced search with a custom query language (boolean logic, field operators, wildcards, regex)
- **File integrity** — SHA-256 hashing and reconciliation for managed files
- **Entity linking** — bidirectional relationships between records
- **Multi-project support** — a registry that tracks multiple databases across the filesystem
- **Organization grouping** — sync records across projects within an organization

This entire directory is designed to be committed to git, making the database portable, diffable, and mergeable.

## Installation

```bash
git clone https://github.com/centy-io/centy-daemon.git
cd centy-daemon
cargo build --release
```

## Usage

### Start the daemon

```bash
# Default: binds to 127.0.0.1:50051
centy-daemon

# Custom address
centy-daemon --addr 127.0.0.1:50052

# Allow additional CORS origins
centy-daemon --cors-origins=http://localhost:5180

# Using environment variables
CENTY_DAEMON_ADDR=127.0.0.1:50052 centy-daemon
```

### gRPC API

The daemon supports both **native gRPC** (HTTP/2) and **gRPC-Web** (HTTP/1.1), making it compatible with:

- Native gRPC clients (CLI tools, backend services)
- Browser-based applications (via gRPC-Web/Connect)

#### Core Operations

See [`proto/centy.proto`](proto/centy.proto) for the full API specification (70+ RPCs).

### CORS Configuration

The daemon always allows CORS requests from:

- All `*.centy.io` subdomains (e.g., `https://app.centy.io`)
- Localhost origins (`http://localhost`, `https://localhost`, `http://127.0.0.1`, `https://127.0.0.1`)

To allow additional custom origins:

```bash
centy-daemon --cors-origins=http://localhost:5180,https://myapp.example.com
```

### Testing the API

```bash
# Install grpcui for a web-based API explorer
go install github.com/fullstorydev/grpcui/cmd/grpcui@latest
grpcui -plaintext 127.0.0.1:50051

# Or use grpcurl for CLI-based interaction
grpcurl -plaintext 127.0.0.1:50051 list
grpcurl -plaintext -d '{"project_path": "/path/to/project"}' \
  127.0.0.1:50051 centy.CentyDaemon/IsInitialized
```

## E2E Tests

```bash
cd e2e
pnpm install
pnpm daemon:build
pnpm daemon:start   # in another terminal
pnpm test
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, and contribution guidelines.

## License

MIT
